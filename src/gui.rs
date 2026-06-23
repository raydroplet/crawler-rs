// src/gui.rs
use crate::app::{
    AppRequest, AppResponse, CrawlCommand, CrawlError, CrawlEvent, CrawlRequest, PageMetadata,
};
use crossbeam_channel as crossbeam;
use eframe;
use egui::{Color32, Pos2, RichText, Shape, Stroke};
use egui_graphs::events::Event;
use egui_graphs::{
    DefaultNodeShape, DisplayEdge, DisplayNode, DrawContext, EdgeProps,
    FruchtermanReingoldWithCenterGravityState, Node,
};
use petgraph::{
    EdgeType, /* Directed, */ Undirected,
    stable_graph::{DefaultIx, IndexType, NodeIndex, StableGraph},
};
use rand::RngExt;
use reqwest::{StatusCode, Url};
use std::collections::{HashMap, HashSet, VecDeque};
use std::f32::consts::TAU;
use std::time::{SystemTime, UNIX_EPOCH};

///////////////////

#[derive(PartialEq)]
enum LeftTab {
    Activity,
    Queue,
    Errors,
}

#[derive(Clone)]
enum NodeData {
    Page(PageMetadata),
    Leaf(Url),
}

struct ActivityMetadata {
    status: StatusCode,
    domain: String,
    path: String,
    timestamp: String, // or chrono::NaiveTime, if you really want to
}

//////////////////

const TABS_ENTRY_COUNT: usize = 128;
struct GraphState {
    is_running: bool,
    show_advanced: bool,
    //
    delta: f32, // dt
    damping: f32,
    max_step: f32,
    epsilon: f32,
    //
    k_scale: f32,
    c_attract: f32,
    c_repulse: f32,
    //
    has_center_gravity: bool,
    center_strenght: f32,
}

////////////////////
/// custom drawing

#[derive(Clone)]
pub struct FixedWidthEdgeShape;

impl<E: Clone> From<EdgeProps<E>> for FixedWidthEdgeShape {
    fn from(_props: EdgeProps<E>) -> Self {
        Self
    }
}

impl<N: Clone, E: Clone, Ty: EdgeType, Ix: IndexType, Dn: DisplayNode<N, E, Ty, Ix>>
    DisplayEdge<N, E, Ty, Ix, Dn> for FixedWidthEdgeShape
{
    fn shapes(
        &mut self,
        start: &Node<N, E, Ty, Ix, Dn>,
        end: &Node<N, E, Ty, Ix, Dn>,
        ctx: &DrawContext<'_>,
    ) -> Vec<Shape> {
        let start_pos = ctx.meta.canvas_to_screen_pos(start.location());
        let end_pos = ctx.meta.canvas_to_screen_pos(end.location());
        let stroke = Stroke::new(0.5, Color32::GRAY);
        vec![Shape::line_segment([start_pos, end_pos], stroke)]
    }

    fn update(&mut self, _state: &EdgeProps<E>) {}

    fn is_inside(
        &self,
        _start: &Node<N, E, Ty, Ix, Dn>,
        _end: &Node<N, E, Ty, Ix, Dn>,
        _pos: Pos2,
    ) -> bool {
        false
    }
}
////////////////////

type CustomGraph =
    egui_graphs::Graph<NodeData, (), Undirected, DefaultIx, DefaultNodeShape, FixedWidthEdgeShape>;
pub struct ViewEgui {
    //
    graph_state: GraphState,
    graph_lookup: HashMap<Url, NodeIndex>,
    graph_event_tx: crossbeam::Sender<Event>,
    graph_event_rx: crossbeam::Receiver<Event>,
    graph: CustomGraph,
    graph_selected_node: Option<NodeIndex>,
    //
    show_markdown_window: bool,
    show_outbound_window: bool,
    show_about_window: bool,
    markdown_text: String,
    markdown_url: Option<Url>,
    //
    left_tab: LeftTab,
    free_graph_movement: bool,
    // Card expansion states
    node_details_expanded: bool,
    hubs_expanded: bool,
    broken_expanded: bool,
    graph_expanded: bool,
    first_crawl: bool,
    //
    show_crawl_window: bool,
    crawl_input_url: String,
    crawl_input_depth: i8,
    //
    app_rx: flume::Receiver<AppResponse>,
    app_tx: flume::Sender<AppRequest>,
    //
    tab_activity_data: VecDeque<ActivityMetadata>,
    tab_queued_data: VecDeque<(usize, Url)>,
    tab_errors_data: VecDeque<(Url, CrawlError)>,
    hubs_data: HashSet<(usize, Url)>,
    broken_data: HashSet<(StatusCode, Url)>,
    info_crawled: usize,
    info_queued: usize,
    info_skipped: usize,
    info_average_sum: u128,
}

impl ViewEgui {
    pub fn new(
        app_response_rx: flume::Receiver<AppResponse>,
        app_request_tx: flume::Sender<AppRequest>,
    ) -> Self {
        let mut graph = CustomGraph::new(StableGraph::default());
        Self::distribute_nodes_circle_generic(&mut graph);

        let state = GraphState {
            is_running: true,
            show_advanced: false,
            //
            delta: 0.100,       // delta: 0.050,
            damping: 0.01,      // damping: 0.30,
            max_step: 3.0,      // max_step: 10.0,
            epsilon: 0.0100000, // epsilon: 0.0010,
            //
            k_scale: 3.0, // k_scale: 1.0,
            c_attract: 1.0,
            c_repulse: 1.0,
            //
            has_center_gravity: true,
            center_strenght: 0.30,
        };

        let (graph_event_tx, graph_event_rx) = crossbeam::unbounded();

        Self {
            graph_state: state,
            graph_lookup: HashMap::new(),
            graph_event_tx: graph_event_tx,
            graph_event_rx: graph_event_rx,
            graph_selected_node: None,
            graph: graph,
            show_markdown_window: false,
            show_outbound_window: false,
            show_about_window: false,
            markdown_text: String::new(),
            markdown_url: None,
            left_tab: LeftTab::Activity,
            free_graph_movement: false,
            node_details_expanded: false,
            hubs_expanded: false,
            broken_expanded: false,
            graph_expanded: false,
            first_crawl: false,
            //
            show_crawl_window: true,
            crawl_input_url: String::from("https://raydroplet.dev/"),
            crawl_input_depth: 0,
            //
            app_rx: app_response_rx,
            app_tx: app_request_tx,
            //
            tab_activity_data: VecDeque::new(),
            tab_queued_data: VecDeque::new(),
            tab_errors_data: VecDeque::new(),
            hubs_data: HashSet::new(),
            broken_data: HashSet::new(),
            //
            info_crawled: 0,
            info_queued: 0,
            info_skipped: 0,
            info_average_sum: 0,
        }
    }

    pub fn run(view: ViewEgui) -> eframe::Result<()> {
        let mut options = eframe::NativeOptions::default();

        options.viewport = egui::ViewportBuilder::default()
            .with_resizable(false)
            .with_inner_size([(1920.0 / 4.0) * 3.0, (1080.0 / 4.0) * 3.0])
            .with_active(false);

        eframe::run_native(
            "crawler-rs",
            options,
            Box::new(|_context| Ok(Box::new(view))),
        )
    }

    fn distribute_nodes_circle_generic(g: &mut CustomGraph) {
        let n_usize = core::cmp::max(g.node_count(), 1);
        if n_usize == 0 {
            return;
        }
        let n_f32 = n_usize as f32;
        let radius = n_f32.sqrt() * 50.0 + 50.0;
        let indices: Vec<_> = g.g().node_indices().collect();
        for (i, idx) in indices.into_iter().enumerate() {
            if let Some(node) = g.g_mut().node_weight_mut(idx) {
                let angle = i as f32 / n_f32 * std::f32::consts::TAU;
                node.set_location(Pos2::new(radius * angle.cos(), radius * angle.sin()));
            }
        }
    }
}

impl ViewEgui {
    pub fn generate_random_petgraph(
        num_nodes: usize,
        num_edges: usize,
    ) -> StableGraph<(), (), Undirected, DefaultIx> {
        let mut rng = rand::rng();
        let mut graph: StableGraph<(), (), Undirected, DefaultIx> = StableGraph::default();

        for _ in 0..num_nodes {
            graph.add_node(());
        }

        for _ in 0..num_edges {
            let source = rng.random_range(0..num_nodes);
            let target = rng.random_range(0..num_nodes);

            graph.add_edge(
                petgraph::stable_graph::NodeIndex::new(source),
                petgraph::stable_graph::NodeIndex::new(target),
                (),
            );
        }

        graph
    }

    fn format_timestamp(timepoint: SystemTime) -> String {
        let Ok(duration) = timepoint.duration_since(UNIX_EPOCH) else {
            return String::from("Time went backwards");
        };
        let total_seconds = duration.as_secs();
        let seconds_today = total_seconds % 86400;
        let hours = seconds_today / 3600;
        let minutes = (seconds_today % 3600) / 60;
        format!("{:02}:{:02}", hours, minutes)
    }

    ////////
    fn poll_channel_events(&mut self) {
        if let Ok(message) = self.app_rx.try_recv() {
            match message {
                AppResponse::Crawler(event) => match event {
                    CrawlEvent::Page(metadata) => {
                        // activity tab.
                        // push element. limit the ammount.
                        let domain = String::from(metadata.url.domain().unwrap_or("None"));
                        let path = String::from(metadata.url.path());
                        let timepoint =
                            String::from(Self::format_timestamp(metadata.timestamp_start));
                        let activity_metadata = ActivityMetadata {
                            status: metadata.status,
                            domain: domain,
                            path: path,
                            timestamp: timepoint,
                        };

                        self.tab_activity_data.push_front(activity_metadata);
                        if self.tab_activity_data.len() > TABS_ENTRY_COUNT {
                            self.tab_activity_data.pop_back();
                        }

                        // hubs card.
                        // inserts into an already sorted vec
                        let item = (metadata.discovered_links.len(), metadata.url.clone());
                        self.hubs_data.insert(item);

                        // markdown window (quick fix)
                        let selected_node_url = self
                            .get_selected_node_payload()
                            .and_then(|data| match data {
                                NodeData::Page(metadata) => Some(&metadata.url),
                                NodeData::Leaf(url) => Some(url),
                        });
                        if Some(&metadata.url) == selected_node_url {
                            println!("update");
                            self.update_markdown_window();
                        }

                        // (the) broken widget
                        if metadata.status.is_server_error() || metadata.status.is_client_error() {
                            self.broken_data
                                .insert((metadata.status, metadata.url.clone()));
                        }

                        // node
                        let metadata_url = metadata.url.clone();
                        let mut parent_pos = Pos2::default();
                        let source_index =
                            if let Some(&index) = self.graph_lookup.get(&metadata_url) {
                                // said page is already present, update it
                                if let Some(node) = self.graph.node_mut(index) {
                                    // TODO: prune orphan edges down below (for now it only adds)
                                    *node.payload_mut() = NodeData::Page(metadata.clone());
                                    parent_pos = node.location();
                                }
                                index
                            } else {
                                // add a root node
                                //
                                // TODO: consider using a custom type instead of reqwest::Url.
                                // returned urls by the crawler must always have valid domains.
                                // unwrap/expect everywhere is redundant.
                                let label = String::from(
                                    metadata.url.domain().expect("domain must always be valid"),
                                );
                                let index = self.graph.add_node_with_label(
                                    NodeData::Page(metadata.clone()),
                                    label.clone(),
                                );
                                self.graph_lookup.insert(metadata_url, index);
                                index
                            };

                        // edges
                        // NOTE: here
                        for link in metadata.discovered_links {
                            let target_index = if let Some(&index) = self.graph_lookup.get(&link) {
                                // get the index of the target node
                                index
                            } else {
                                // jitter
                                let mut rng = rand::rng();
                                let angle = rng.random_range(0.0..TAU);
                                let distance = rng.random_range(10.0..50.0);
                                let jitter_x = angle.cos() * distance;
                                let jitter_y = angle.sin() * distance;

                                // if said node doesn't exist, create an empty one
                                let location: Pos2 =
                                    Pos2::new(parent_pos.x + jitter_x, parent_pos.y + jitter_y);
                                let label = String::from(
                                    link.domain().expect("domain must always be valid"),
                                );
                                let data = NodeData::Leaf(link.clone());
                                let index = self.graph.add_node_with_label_and_location(
                                    data,
                                    label.clone(),
                                    location,
                                );
                                self.graph_lookup.insert(link, index);

                                index
                            };

                            // finally link the nodes
                            if self
                                .graph
                                .edges_connecting(source_index, target_index)
                                .next()
                                .is_none()
                            {
                                self.graph.add_edge(source_index, target_index, ());
                            }
                        }

                        // info panel
                        self.info_crawled += 1;
                        self.info_average_sum += metadata
                            .timestamp_end
                            .duration_since(metadata.timestamp_start)
                            .expect("time went backwards")
                            .as_millis();
                    }
                    CrawlEvent::Queued(url, count) => {
                        // queued tab.
                        // push element. limit the ammount.
                        let item = (count, url);
                        self.tab_queued_data.push_front(item);
                        if self.tab_queued_data.len() > TABS_ENTRY_COUNT {
                            self.tab_queued_data.pop_back();
                        }
                        self.info_queued += 1;
                    }
                    CrawlEvent::Skipped(_url) => {
                        self.info_skipped += 1;
                    }
                    CrawlEvent::Error(url, error) => {
                        self.tab_errors_data.push_front((url, error));
                        if self.tab_errors_data.len() > TABS_ENTRY_COUNT {
                            self.tab_errors_data.pop_back();
                        }
                    }
                },
                AppResponse::Markdown(url, content) => {
                    self.markdown_text = content;
                    self.markdown_url = Some(url);
                }
            }
        }
    }

    fn widget_url(ui: &mut egui::Ui, url: &Url, vertical: bool) {
        let fancy_url = |ui: &mut egui::Ui| {
            let mut text =
                egui::RichText::new(url.domain().expect("url domain must always be valid"))
                    .size(12.0);

            if vertical {
                text = text.strong();
            }

            ui.add(egui::Label::new(text).truncate());

            // mute the path so the domain stands out as the primary identifier
            ui.add({
                egui::Label::new(
                    egui::RichText::new(url.path())
                        .color(ui.visuals().weak_text_color())
                        .size(10.0),
                )
                .truncate()
            });
        };

        if !vertical {
            ui.with_layout(egui::Layout::left_to_right(egui::Align::Center), |ui| {
                ui.spacing_mut().item_spacing.x = 0.0;
                fancy_url(ui);
            });
        } else {
            ui.vertical_centered(|ui| {
                fancy_url(ui);
            });
        }
    }

    fn update_markdown_window(&mut self) {
        if !self.show_markdown_window {
            return;
        }

        // get the current node url
        let url_opt = self.get_selected_node_payload().map(|data| match data {
            NodeData::Page(metadata) => metadata.url.clone(),
            NodeData::Leaf(url) => url.clone(),
        });
        if let Some(url) = url_opt {
            println!("markdown request sent for {}", url.clone());
            let _ = self.app_tx.send(AppRequest::Markdown(url));
        }
    }

    fn get_selected_node_payload(&self) -> Option<&NodeData> {
        self.graph_selected_node
            .and_then(|index| self.graph.node(index))
            .map(|node| node.payload())
    }

    fn poll_graph_events(&mut self) {
        while let Ok(event) = self.graph_event_rx.try_recv() {
            match event {
                Event::NodeClick(payload) => {
                    self.graph_selected_node = Some(NodeIndex::new(payload.id));
                    self.update_markdown_window();
                    println!("Node {:?} was clicked", payload);
                }
                // catch-all for other events like pan, zoom, or edge selections
                _ => {}
            }
        }
    }

    fn render_top_menu(&mut self, ui: &mut egui::Ui, menu_frame: egui::Frame) {
        // TODO: assert if this is centered correctly
        egui::Panel::top("top_menu_bar")
            .frame(menu_frame)
            .show_inside(ui, |ui| {
                let available_width = ui.available_width();

                // estimates the width of menu items.
                // you may need to tweak this number based on your font size and labels.
                // "File" + "View" + "Graph" + "..."
                let estimated_menu_width = 200.0;

                // calculate the padding needed on the left to center it
                let left_padding = (available_width - estimated_menu_width) / 2.0;

                ui.horizontal(|ui| {
                    // push the menu bar to the right by adding empty space
                    if left_padding > 0.0 {
                        ui.add_space(left_padding);
                    }

                    // draw the menu bar
                    egui::MenuBar::new().ui(ui, |ui| {
                        ui.menu_button("File", |ui| {
                            if ui.button("❌ Exit").clicked() {
                                ui.send_viewport_cmd(egui::ViewportCommand::Close);
                            }
                        });

                        ui.menu_button("View", |ui| {
                            if ui.button("🔍 Toggle Panels").clicked() {
                                self.node_details_expanded = !self.node_details_expanded;
                                self.graph_expanded = !self.graph_expanded;
                                self.hubs_expanded = !self.hubs_expanded;
                                self.broken_expanded = !self.broken_expanded;
                            }
                        });

                        ui.menu_button("Graph", |ui| {
                            let message = if self.free_graph_movement {
                                "Lock"
                            } else {
                                "Unlock"
                            };
                            if ui.button(message).clicked() {
                                self.free_graph_movement = !self.free_graph_movement;
                                println!("Center {}", message);
                            }
                            if ui.button("Reorganize").clicked() {
                                Self::distribute_nodes_circle_generic(&mut self.graph);
                                println!("Reorganize");
                            }
                        });
                        if ui.button("Crawl").clicked() {
                            self.show_crawl_window = !self.show_crawl_window;
                            if self.show_crawl_window {
                                self.show_about_window = false;
                            }
                        };
                        if ui.button("About").clicked() {
                            self.show_about_window = !self.show_about_window;
                            if self.show_about_window {
                                self.show_crawl_window = false; // out of the way!
                            }
                        };
                    });
                });
            });
    }

    fn render_central_panel(
        &mut self,
        ui: &mut egui::Ui,
        graph_frame: egui::Frame,
        center_offset: f32,
    ) {
        egui::CentralPanel::default()
            .frame(graph_frame)
            .show_inside(ui, |ui| {
                let central_rect = ui.max_rect();

                if self.show_markdown_window {
                    egui::Window::new("Markdown Source")
                        .open(&mut self.show_markdown_window)
                        .resizable(true)
                        .collapsible(false)
                        .constrain_to(central_rect)
                        .default_size([500.0, 400.0])
                        .show(ui.ctx(), |ui| {
                            // check if a node is selected (no node selected)
                            // check if the node selected is equal to the markdown we have (not queried)

                            let display_text = self
                                .graph_selected_node
                                .and_then(|index| self.graph.node(index))
                                .map(|node| match node.payload() {
                                    NodeData::Page(metadata) => {
                                        Some(&metadata.url) == self.markdown_url.as_ref()
                                    }
                                    NodeData::Leaf(url) => Some(url) == self.markdown_url.as_ref(),
                                });

                            match display_text {
                                Some(node_matches) => {
                                    if node_matches {
                                        egui::ScrollArea::vertical().show(ui, |ui| {
                                            ui.add(
                                                egui::TextEdit::multiline(&mut self.markdown_text)
                                                    .font(egui::TextStyle::Monospace)
                                                    .code_editor()
                                                    .interactive(false)
                                                    .desired_width(f32::INFINITY),
                                            );
                                        });
                                    } else {
                                        ui.label("Invalid node selection.");
                                    }
                                }
                                None => {
                                    ui.label("No node selected.");
                                }
                            }
                        });
                }

                let links_opt = self
                    .get_selected_node_payload()
                    .and_then(|data| match data {
                        NodeData::Page(metadata) => Some(&metadata.discovered_links),
                        NodeData::Leaf(_url) => None,
                    });

                let mut show_outbound_window = self.show_outbound_window;
                egui::Window::new("Outbound Links")
                    .open(&mut show_outbound_window)
                    .resizable(true)
                    .collapsible(false)
                    .constrain_to(central_rect)
                    .default_size([300.0, 200.0])
                    .show(ui.ctx(), |ui| {
                        if let Some(links) = links_opt {
                            if links.len() > 0 {
                                egui::ScrollArea::vertical().show(ui, |ui| {
                                    for link in links {
                                        ui.horizontal(|ui| {
                                            Self::widget_url(ui, link, false);
                                        });
                                    }
                                });
                            } else {
                                ui.label("Page contains no links.");
                            }
                        } else {
                            ui.label("Invalid node selection.");
                        }
                    });
                self.show_outbound_window = show_outbound_window;

                ////////
                let settings_navigation = &egui_graphs::SettingsNavigation::new()
                    .with_zoom_and_pan_enabled(self.free_graph_movement)
                    .with_fit_to_screen_enabled(!self.free_graph_movement);

                let settings_interactions = &egui_graphs::SettingsInteraction::new() //
                    .with_node_clicking_enabled(true);

                let mut view = egui_graphs::GraphView::<
                    _,
                    _,
                    _,
                    _,
                    _,
                    _,
                    FruchtermanReingoldWithCenterGravityState,
                    egui_graphs::LayoutForceDirected<
                        egui_graphs::FruchtermanReingoldWithCenterGravity,
                    >,
                >::new(&mut self.graph)
                .with_navigations(settings_navigation)
                .with_interactions(settings_interactions)
                .with_event_sink(&self.graph_event_tx);

                // 1. Trick the center calculation by expanding the LEFT boundary off-screen.
                let mut virtual_rect = ui.available_rect_before_wrap();
                virtual_rect.max.x -= center_offset;

                // 2. Create the child UI with this shifted virtual rectangle
                let mut graph_ui = ui.new_child(
                    egui::UiBuilder::new()
                        .max_rect(virtual_rect)
                        .layout(*ui.layout()),
                );

                // 3. Constrain the visual drawing back to the actual central panel bounds
                // This ensures the off-screen left expansion doesn't visually bleed
                // over your left side-panel (Activity/Queue/Errors).
                graph_ui.set_clip_rect(ui.clip_rect());

                // 4. Render the graph
                graph_ui.add(&mut view);
                ////////

                /////////////////// MINIMAP
                let minimap_margin = 8.0;
                egui::Window::new("Minimap Overlay")
                    .anchor(
                        egui::Align2::RIGHT_BOTTOM,
                        [minimap_margin - 16.0, -minimap_margin],
                    )
                    .resizable(false)
                    .collapsible(false)
                    .constrain_to(central_rect)
                    .title_bar(false)
                    .frame(egui::Frame::window(&ui.style()).inner_margin(4.0))
                    .show(ui.ctx(), |ui| {
                        let minimap_size = egui::vec2(150.0, 150.0);
                        let (response, painter) =
                            ui.allocate_painter(minimap_size, egui::Sense::hover());

                        painter.rect_filled(response.rect, 4.0, ui.visuals().extreme_bg_color);

                        let mut min_pos = egui::Pos2::new(f32::INFINITY, f32::INFINITY);
                        let mut max_pos = egui::Pos2::new(f32::NEG_INFINITY, f32::NEG_INFINITY);

                        for idx in self.graph.g().node_indices() {
                            if let Some(node) = self.graph.g().node_weight(idx) {
                                let loc = node.location();
                                min_pos = min_pos.min(loc);
                                max_pos = max_pos.max(loc);
                            }
                        }

                        if min_pos.x == f32::INFINITY {
                            min_pos = egui::Pos2::ZERO;
                            max_pos = egui::Pos2::ZERO;
                        }

                        let padding = 20.0;
                        min_pos -= egui::vec2(padding, padding);
                        max_pos += egui::vec2(padding, padding);

                        let graph_size = max_pos - min_pos;
                        let scale = if graph_size.x > 0.0 && graph_size.y > 0.0 {
                            (minimap_size.x / graph_size.x).min(minimap_size.y / graph_size.y)
                        } else {
                            1.0
                        };

                        let graph_center = min_pos.to_vec2() + graph_size / 2.0;
                        let offset = response.rect.center().to_vec2() - (graph_center * scale);

                        let transform = |pos: egui::Pos2| -> egui::Pos2 {
                            egui::Pos2::new(pos.x * scale, pos.y * scale) + offset
                        };

                        for edge_idx in self.graph.g().edge_indices() {
                            if let Some((src, dst)) = self.graph.g().edge_endpoints(edge_idx) {
                                if let (Some(s_node), Some(d_node)) = (
                                    self.graph.g().node_weight(src),
                                    self.graph.g().node_weight(dst),
                                ) {
                                    painter.line_segment(
                                        [
                                            transform(s_node.location()),
                                            transform(d_node.location()),
                                        ],
                                        egui::Stroke::new(1.0, egui::Color32::from_gray(60)),
                                    );
                                }
                            }
                        }

                        for idx in self.graph.g().node_indices() {
                            if let Some(node) = self.graph.g().node_weight(idx) {
                                let color = if node.selected() {
                                    egui::Color32::WHITE
                                } else {
                                    egui::Color32::from_rgb(80, 200, 150)
                                };

                                painter.circle_filled(transform(node.location()), 1.5, color);
                            }
                        }
                    });
            });
    }

    fn render_about_window(&mut self, ui: &mut egui::Ui) {
        egui::Window::new("About")
            .open(&mut self.show_about_window)
            .collapsible(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .resizable(false)
            .show(ui, |ui| {
                ui.vertical_centered(|ui| {
                    ui.heading("Crawler-rs");
                    ui.label(RichText::new("version 0.1.0").weak());
                });
                ui.separator();
                ui.label("This application does awesome things.");
                ui.horizontal(|ui| {
                    ui.label("Source code:");
                    ui.hyperlink_to("GitHub", "https://github.com/raydroplet/crawler-rs");
                });
            });
    }

    fn render_crawl_window(&mut self, ui: &mut egui::Ui) {
        let mut should_close = false;
        egui::Window::new("Crawl")
            .open(&mut self.show_crawl_window)
            .collapsible(false)
            .resizable(false)
            .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
            .show(ui.ctx(), |ui| {
                egui::Grid::new("crawl_input_grid")
                    .num_columns(2)
                    .spacing([10.0, 10.0])
                    .show(ui, |ui| {
                        ui.label("URL:");
                        ui.text_edit_singleline(&mut self.crawl_input_url);
                        ui.end_row();

                        ui.label("Depth:");
                        ui.add(egui::DragValue::new(&mut self.crawl_input_depth).range(0..=1));
                        ui.end_row();
                    });

                ui.add_space(12.0);

                ui.horizontal(|ui| {
                    if ui.button("🕷 Crawl Page").clicked() {
                        let Ok(url) = Url::parse(&self.crawl_input_url) else {
                            println!("failed to parse"); // TODO: provide a better warning
                            return;
                        };

                        let request = CrawlRequest {
                            source: url,
                            depth: self.crawl_input_depth,
                        };
                        let command = CrawlCommand::Request(request);
                        let _ = self.app_tx.send(AppRequest::Crawler(command));

                        if !self.first_crawl {
                            self.node_details_expanded = true;
                            self.hubs_expanded = true;
                            self.broken_expanded = true;
                            self.graph_expanded = true;
                            self.first_crawl = true;
                        }

                        should_close = true;
                    }

                    ui.add_enabled_ui(false, |ui| {
                        if ui.button("⏹ Terminate").clicked() {
                            let command = CrawlCommand::Terminate;
                            let _ = self.app_tx.send(AppRequest::Crawler(command));
                        };
                    });
                });
            });

        if should_close {
            self.show_crawl_window = false;
        }
    }

    fn render_right_window(
        &mut self,
        ui: &mut egui::Ui,
        margin_offset: i8,
        anchor_y_offset: f32,
        width: f32,
        minimap_reserved_height: f32,
        cards_spacing: f32,
    ) {
        let down_triangle_icon = "🔻";
        let left_triangle_icon = "◀";
        let margin = egui::Margin::same(margin_offset);
        //
        fn add_stretched_right_cell(ui: &mut egui::Ui, content: impl FnOnce(&mut egui::Ui)) {
            ui.scope(|ui| {
                ui.set_min_width(ui.available_width());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), content);
            });
        }
        //
        egui::Window::new("Right_Inspector_Window")
            // Anchor to top right, with a 10px margin off the edges
            .anchor(egui::Align2::RIGHT_TOP, [0.0, anchor_y_offset]) // WARN: y expected to match the top bar height
            .resizable(false)
            .collapsible(false)
            .title_bar(false) // Hides the drag-bar so it looks like a built-in UI panel
            .frame(
                egui::Frame::NONE
                    .fill(Color32::from_black_alpha(200)) // Slight transparency looks great over graphs
                    .corner_radius(0.0)
                    .inner_margin(margin),
            )
            .show(ui, |ui| {
                ui.set_width(width); // Force the width

                // Calculate the maximum allowed height so it doesn't overlap the minimap
                let max_height = ui.ctx().content_rect().height()
                    - anchor_y_offset
                    - (margin_offset as f32)
                    - minimap_reserved_height;

                ui.set_max_height(max_height);
                egui::ScrollArea::vertical()
                    // [horizontal, vertical]: Allow vertical shrinking so it hugs the cards
                    .auto_shrink([false, true])
                    .show(ui, |ui| {
                        let card_frame = egui::Frame::default()
                            .fill(ui.visuals().window_fill)
                            .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                            .inner_margin(egui::Margin::same(12))
                            .corner_radius(0.0);

                        // --- SECTION 1: NODE ---
                        card_frame.show(ui, |ui| {
                            ui.set_min_width(ui.available_width()); // WARN: what this line does?

                            ui.horizontal(|ui| {
                                ui.heading("⏺ Node");
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let icon = if self.node_details_expanded {
                                            down_triangle_icon
                                        } else {
                                            left_triangle_icon
                                        };
                                        if ui.add(egui::Button::new(icon).frame(false)).clicked() {
                                            self.node_details_expanded =
                                                !self.node_details_expanded;
                                        }
                                    },
                                );
                            });

                            if self.node_details_expanded {
                                ui.add_space(4.0);
                                ui.separator();
                                ui.add_space(4.0);

                                ui.set_min_width(ui.available_width());

                                if let Some(index) = self.graph_selected_node {
                                    let node = self
                                        .graph
                                        .node(index)
                                        .expect("graph_selected_node must always be a valid index");
                                    match node.payload() {
                                        NodeData::Page(metadata) => {
                                            // url header
                                            egui::Frame::NONE
                                                .fill(ui.visuals().faint_bg_color)
                                                .corner_radius(6.0)
                                                .inner_margin(8.0)
                                                .show(ui, |ui| {
                                                    Self::widget_url(ui, &metadata.url, true);
                                                });
                                            ui.add_space(8.0);

                                            let code_color = if metadata.status.is_success() {
                                                Color32::from_rgb(0, 100, 0)
                                            } else if metadata.status.is_client_error()
                                                || metadata.status.is_server_error()
                                            {
                                                Color32::from_rgb(100, 0, 0)
                                            } else {
                                                Color32::from_rgb(100, 100, 0)
                                            };

                                            egui::Grid::new("node_details_grid")
                                                .num_columns(2)
                                                .striped(true)
                                                .spacing([20.0, 4.0])
                                                .show(ui, |ui| {
                                                    ui.label("Status");
                                                    add_stretched_right_cell(ui, |ui| {
                                                        ui.label(
                                                            RichText::new(format!(
                                                                " {} ",
                                                                metadata.status.as_u16()
                                                            ))
                                                            .background_color(code_color)
                                                            .color(Color32::WHITE),
                                                        );
                                                    });
                                                    ui.end_row();

                                                    ui.label("Links out");
                                                    add_stretched_right_cell(ui, |ui| {
                                                        ui.label(format!(
                                                            "{}",
                                                            metadata.discovered_links.len()
                                                        ));
                                                    });
                                                    ui.end_row();

                                                    // ui.label("Page size");
                                                    // add_stretched_right_cell(ui, |ui| {
                                                    //     ui.label("62 KB");
                                                    // });
                                                    // ui.end_row();

                                                    let load_time_ms = metadata
                                                        .timestamp_end
                                                        .duration_since(metadata.timestamp_start)
                                                        .expect("time went backwards")
                                                        .as_millis();
                                                    ui.label("Crawl time");
                                                    add_stretched_right_cell(ui, |ui| {
                                                        ui.label(format!("{}ms", load_time_ms));
                                                    });
                                                    ui.end_row();
                                                });

                                            ui.add_space(8.0);

                                            ui.horizontal(|ui| {
                                                if ui.button("📝 Markdown").clicked() {
                                                    self.show_markdown_window =
                                                        !self.show_markdown_window;

                                                    self.update_markdown_window();
                                                }
                                                if ui.button("🕸 Links").clicked() {
                                                    self.show_outbound_window =
                                                        !self.show_outbound_window;
                                                }
                                            });
                                        }
                                        NodeData::Leaf(url) => {
                                            egui::Frame::NONE
                                                .fill(ui.visuals().faint_bg_color)
                                                .corner_radius(6.0)
                                                .inner_margin(8.0)
                                                .show(ui, |ui| {
                                                    Self::widget_url(ui, &url, true);
                                                });
                                            ui.vertical_centered(|ui| {
                                                ui.add_space(8.0);
                                                ui.label("Page not crawled");
                                                ui.separator();
                                                ui.add_space(8.0);
                                                if ui.button("Crawl Page").clicked() {
                                                    let request = CrawlRequest {
                                                        source: url.clone(),
                                                        depth: 0,
                                                    };
                                                    let command = CrawlCommand::Request(request);
                                                    let _ = self
                                                        .app_tx
                                                        .send(AppRequest::Crawler(command));
                                                }
                                            });
                                        }
                                    }
                                } else {
                                    ui.label("No node selected");
                                }
                            }
                        });

                        ui.add_space(cards_spacing);

                        card_frame.show(ui, |ui| {
                            ui.horizontal(|ui| {
                                ui.heading("⬣ Graph");
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let icon = if self.graph_expanded {
                                            down_triangle_icon
                                        } else {
                                            left_triangle_icon
                                        };
                                        if ui.add(egui::Button::new(icon).frame(false)).clicked() {
                                            self.graph_expanded = !self.graph_expanded;
                                        }
                                    },
                                );
                            });

                            if self.graph_expanded {
                                ui.add_space(4.0);
                                ui.separator();
                                ui.add_space(4.0);
                                //
                                let spacing = ui.spacing_mut();
                                spacing.slider_width = 60.0; // Adjust this number until it fits your card
                                //
                                egui::Grid::new("graph_details_grid")
                                    .num_columns(2)
                                    .striped(true)
                                    .spacing([20.0, 4.0])
                                    .show(ui, |ui| {
                                        ui.label("Nodes");
                                        add_stretched_right_cell(ui, |ui| {
                                            ui.label(self.graph.node_count().to_string());
                                        });
                                        ui.end_row();
                                        //
                                        ui.label("Links");
                                        add_stretched_right_cell(ui, |ui| {
                                            ui.label(self.graph.edge_count().to_string());
                                        });
                                        ui.end_row();
                                        //
                                    });

                                ui.add_space(8.0);

                                ui.horizontal(|ui| {
                                    let message = if self.free_graph_movement {
                                        "🎯 Lock"
                                    } else {
                                        "🎯 Unlock"
                                    };
                                    if ui.button("🔀 Reorganize").clicked() {
                                        Self::distribute_nodes_circle_generic(&mut self.graph);
                                    }
                                    if ui.button(message).clicked() {
                                        self.free_graph_movement = !self.free_graph_movement;
                                    }
                                });

                                ui.separator();
                                ///////////////
                                if self.graph_state.is_running {
                                    // ui.add_space(8.0);

                                    let mut state = egui_graphs::get_layout_state::<
                                        FruchtermanReingoldWithCenterGravityState,
                                    >(ui, None);

                                    if self.graph_state.show_advanced {
                                        egui::Grid::new("physics_sliders_grid")
                                            .num_columns(2)
                                            .spacing([8.0, 4.0]) // Adjust horizontal/vertical spacing as needed
                                            .show(ui, |ui| {
                                                ui.label("Delta");
                                                ui.add(egui::Slider::new(
                                                    &mut self.graph_state.delta,
                                                    0.001..=0.2,
                                                ));
                                                ui.end_row();

                                                ui.label("Damping");
                                                ui.add(egui::Slider::new(
                                                    &mut self.graph_state.damping,
                                                    0.0..=1.0,
                                                ));
                                                ui.end_row();

                                                ui.label("Max Step");
                                                ui.add(egui::Slider::new(
                                                    &mut self.graph_state.max_step,
                                                    0.1..=50.0,
                                                ));
                                                ui.end_row();

                                                ui.label("Epsilon");
                                                ui.add(
                                                    egui::Slider::new(
                                                        &mut self.graph_state.epsilon,
                                                        1e-5..=1e-1,
                                                    )
                                                    .logarithmic(true)
                                                    .max_decimals(3),
                                                );
                                                ui.end_row();

                                                ui.label("K Scale");
                                                ui.add(egui::Slider::new(
                                                    &mut self.graph_state.k_scale,
                                                    0.2..=3.0,
                                                ));
                                                ui.end_row();

                                                ui.label("C Attract");
                                                ui.add(egui::Slider::new(
                                                    &mut self.graph_state.c_attract,
                                                    0.1..=3.0,
                                                ));
                                                ui.end_row();

                                                ui.label("C Repulse");
                                                ui.add(egui::Slider::new(
                                                    &mut self.graph_state.c_repulse,
                                                    0.1..=3.0,
                                                ));
                                                ui.end_row();

                                                // TODO: consider adding a organize type dropdown to
                                                // select the function to use when distributing nodes
                                            });

                                        ui.add_space(8.0);
                                    }

                                    ui.horizontal(|ui| {
                                        ui.checkbox(&mut self.graph_state.is_running, "Animated");
                                        ui.checkbox(
                                            &mut self.graph_state.show_advanced,
                                            "Advanced",
                                        );
                                    });

                                    // overwrite widget values
                                    state.base.is_running = self.graph_state.is_running;
                                    state.base.dt = self.graph_state.delta;
                                    state.base.damping = self.graph_state.damping;
                                    state.base.max_step = self.graph_state.max_step;
                                    state.base.epsilon = self.graph_state.epsilon;
                                    state.base.k_scale = self.graph_state.k_scale;
                                    state.base.c_attract = self.graph_state.c_attract;
                                    state.base.c_repulse = self.graph_state.c_repulse;
                                    state.extras.0.enabled = self.graph_state.has_center_gravity;
                                    state.extras.0.params.c = self.graph_state.center_strenght;

                                    egui_graphs::set_layout_state::<
                                        FruchtermanReingoldWithCenterGravityState,
                                    >(ui, state, None);
                                } else {
                                    ui.checkbox(&mut self.graph_state.is_running, "Animated");
                                }
                                ///////////////
                            }
                        });

                        ui.add_space(cards_spacing);

                        // --- SECTION 2: HUBS ---
                        card_frame.show(ui, |ui| {
                            ui.set_min_width(ui.available_width());

                            ui.horizontal(|ui| {
                                ui.heading("🌐 Hubs");
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let icon = if self.hubs_expanded {
                                            down_triangle_icon
                                        } else {
                                            left_triangle_icon
                                        };
                                        if ui.add(egui::Button::new(icon).frame(false)).clicked() {
                                            self.hubs_expanded = !self.hubs_expanded;
                                        }
                                    },
                                );
                            });

                            if self.hubs_expanded && self.hubs_data.len() > 0 {
                                ui.add_space(4.0);
                                ui.separator();
                                ui.add_space(4.0);

                                egui::ScrollArea::vertical()
                                    .id_salt("hubs_scroll_area")
                                    .max_height(200.0)
                                    .show(ui, |ui| {
                                        for (value, url) in self.hubs_data.iter() {
                                            ui.horizontal(|ui| {
                                                ui.vertical(|ui| {
                                                    ui.horizontal(|ui| {
                                                        ui.with_layout(
                                                            egui::Layout::right_to_left(
                                                                egui::Align::Center,
                                                            ),
                                                            |ui| {
                                                                ui.label(
                                                                    egui::RichText::new(format!(
                                                                        "{}",
                                                                        value
                                                                    ))
                                                                    .color(egui::Color32::GRAY),
                                                                );
                                                                //
                                                                ui.spacing_mut().item_spacing.x =
                                                                    0.0;
                                                                ui.with_layout(
                                                                    egui::Layout::left_to_right(
                                                                        egui::Align::Center,
                                                                    ),
                                                                    |ui| {
                                                                        Self::widget_url(
                                                                            ui, &url, false,
                                                                        );
                                                                    },
                                                                );
                                                            },
                                                        );
                                                    });

                                                    let progress = (*value as f32
                                                        / self
                                                            .hubs_data
                                                            .iter()
                                                            .next()
                                                            .expect("Lenght check already done")
                                                            .0
                                                            as f32)
                                                        .clamp(0.0, 1.0);
                                                    ui.add(
                                                        egui::ProgressBar::new(progress)
                                                            .desired_height(3.0)
                                                            .fill(egui::Color32::from_rgb(
                                                                40, 100, 180,
                                                            )),
                                                    );
                                                });
                                            });
                                            ui.add_space(4.0);
                                        }
                                    });
                            }
                        });

                        ui.add_space(cards_spacing);

                        // --- SECTION 3: BROKEN ---
                        card_frame.show(ui, |ui| {
                            ui.set_min_width(ui.available_width());

                            ui.horizontal(|ui| {
                                ui.heading("❌ Broken");
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| {
                                        let icon = if self.broken_expanded {
                                            down_triangle_icon
                                        } else {
                                            left_triangle_icon
                                        };
                                        if ui.add(egui::Button::new(icon).frame(false)).clicked() {
                                            self.broken_expanded = !self.broken_expanded;
                                        }
                                    },
                                );
                            });

                            if self.broken_expanded && self.broken_data.len() > 0 {
                                ui.add_space(4.0);
                                ui.separator();
                                ui.add_space(4.0);

                                for (status, url) in self.broken_data.iter() {
                                    ui.horizontal(|ui| {
                                        ui.vertical(|ui| {
                                            ui.horizontal(|ui| {
                                                ui.with_layout(
                                                    egui::Layout::right_to_left(
                                                        egui::Align::Center,
                                                    ),
                                                    |ui| {
                                                        ui.label(
                                                            RichText::new(format!(
                                                                " {} ",
                                                                status.as_u16()
                                                            ))
                                                            .background_color(Color32::from_rgb(
                                                                80, 10, 10,
                                                            ))
                                                            .color(Color32::WHITE),
                                                        );
                                                        ui.spacing_mut().item_spacing.x = 0.0;
                                                        Self::widget_url(ui, url, false);
                                                    },
                                                );
                                            });
                                        });
                                    });
                                }
                            }
                        });
                    });
            });
    }

    fn render_left_panel(
        &mut self,
        ui: &mut egui::Ui,
        panel_frame: egui::Frame,
        default_size: f32,
    ) {
        egui::Panel::left("left_crawling_input")
            .frame(panel_frame.clone())
            .resizable(true)
            .default_size(default_size)
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.left_tab, LeftTab::Activity, "📈 Activity");
                    ui.selectable_value(&mut self.left_tab, LeftTab::Queue, "⏳ Queued");
                    ui.selectable_value(&mut self.left_tab, LeftTab::Errors, "❌ Errors");
                });

                ui.separator();

                egui::Panel::bottom("left_metrics_footer")
                    .frame(egui::Frame::NONE)
                    .show_inside(ui, |ui| {
                        ui.add_space(8.0);

                        ui.columns(2, |cols| {
                            cols[0].group(|ui| {
                                ui.vertical_centered_justified(|ui| {
                                    ui.heading(
                                        RichText::new(self.info_crawled.to_string()), /* .color(Color32::LIGHT_BLUE) */
                                    );
                                    ui.label(RichText::new("Crawled").weak());
                                });
                            });
                            cols[1].group(|ui| {
                                ui.vertical_centered_justified(|ui| {
                                    ui.heading(
                                        RichText::new(self.info_queued.to_string()), /* .color(Color32::YELLOW) */
                                    );
                                    ui.label(RichText::new("Queued").weak());
                                });
                            });
                        });

                        ui.add_space(4.0);

                        let avg = self.info_average_sum
                            .checked_div(self.info_crawled as u128)
                            .unwrap_or(0);
                        ui.columns(2, |cols| {
                            cols[0].group(|ui| {
                                ui.vertical_centered_justified(|ui| {
                                    ui.heading(
                                        RichText::new(self.info_skipped.to_string()), /* .color(Color32::LIGHT_RED) */
                                    );
                                    ui.label(RichText::new("Skipped").weak());
                                });
                            });
                            cols[1].group(|ui| {
                                ui.vertical_centered_justified(|ui| {
                                    ui.heading(format!("{avg:.0}ms"));
                                    ui.label(RichText::new("Average").weak());
                                });
                            });
                        });

                        ui.add_space(4.0);
                    });

                egui::CentralPanel::default()
                    .frame(egui::Frame::NONE)
                    .show_inside(ui, |ui| match self.left_tab {
                        LeftTab::Activity => {
                            egui::Frame::NONE
                                .fill(ui.visuals().extreme_bg_color)
                                .inner_margin(egui::Margin::same(4))
                                .show(ui, |ui| {
                                    egui::ScrollArea::vertical()
                                        .auto_shrink([false, false]) // Forces scroll area to fill the panel height
                                        .show(ui, |ui| {
                                            for entry in self.tab_activity_data.iter() {
                                                let status = entry.status;

                                                ui.horizontal(|ui| {
                                                    let code_color = if status.is_success() {
                                                        Color32::from_rgb(0, 100, 0)
                                                    } else if status.is_client_error()
                                                        || status.is_server_error()
                                                    {
                                                        Color32::from_rgb(100, 0, 0)
                                                    } else {
                                                        Color32::from_rgb(100, 100, 0)
                                                    };

                                                    // 1. Claim space on the left (Badge)
                                                    ui.label(
                                                        RichText::new(format!(
                                                            " {} ",
                                                            status.as_u16()
                                                        ))
                                                        .background_color(code_color)
                                                        .color(Color32::WHITE)
                                                    );

                                                    // 2. Anchor the rest of the layout to the right
                                                    ui.with_layout(
                                                        egui::Layout::right_to_left(
                                                            egui::Align::Center,
                                                        ),
                                                        |ui| {
                                                            ui.label(
                                                                RichText::new(
                                                                    entry.timestamp.clone(),
                                                                )
                                                                .color(Color32::DARK_GRAY),
                                                            );

                                                            // 4. Fill the remaining middle space with the truncated address
                                                            let url = Url::parse(&format!("https://{}{}", entry.domain, entry.path))
                                                                .expect("domain must always be valid.");
                                                            Self::widget_url(ui, &url, false);
                                                        },
                                                    );
                                                });
                                            }
                                        });
                                });
                        }

                        LeftTab::Queue => {
                            egui::Frame::NONE
                                .fill(ui.visuals().extreme_bg_color)
                                .inner_margin(egui::Margin::same(4))
                                .show(ui, |ui| {
                                    egui::ScrollArea::vertical()
                                        .auto_shrink([false, false])
                                        .show(ui, |ui| {
                                            for (count, url) in &self.tab_queued_data {
                                                ui.horizontal(|ui| {
                                                    // Depth Badge (Blue)
                                                    ui.label(
                                                        RichText::new(format!(" {} ", count))
                                                            .background_color(Color32::from_rgb(
                                                                60, 60, 60,
                                                            ))
                                                            .color(Color32::WHITE),
                                                    );

                                                    ui.with_layout(
                                                        egui::Layout::right_to_left(
                                                            egui::Align::Center,
                                                        ),
                                                        |ui| {
                                                            // Using a simple indicator instead of time, as it hasn't been crawled yet
                                                            ui.label(
                                                                RichText::new("⏳")
                                                                    .color(Color32::DARK_GRAY),
                                                            );

                                                            ui.with_layout(
                                                                egui::Layout::left_to_right(
                                                                    egui::Align::Center,
                                                                ),
                                                                |ui| {
                                                                    ui.spacing_mut()
                                                                        .item_spacing
                                                                        .x = 0.0;
                                                                    ui.label(url.domain().expect(
                                                                        "domain must be valid",
                                                                    ));
                                                                    let weak_color = ui
                                                                        .visuals()
                                                                        .weak_text_color();
                                                                    ui.visuals_mut()
                                                                        .override_text_color =
                                                                        Some(weak_color);
                                                                    ui.add(
                                                                        egui::Label::new(
                                                                            url.path(),
                                                                        )
                                                                        .truncate(),
                                                                    );
                                                                },
                                                            );
                                                        },
                                                    );
                                                });
                                            }
                                        });
                                });
                        }

                        LeftTab::Errors => {
                            egui::Frame::NONE
                                .fill(ui.visuals().extreme_bg_color)
                                .inner_margin(egui::Margin::same(4))
                                .show(ui, |ui| {
                                    egui::ScrollArea::vertical()
                                        .auto_shrink([false, false])
                                        .show(ui, |ui| {
                                            for (url, error) in &self.tab_errors_data {
                                                ui.horizontal(|ui| {
                                                    // Generic Error Badge
                                                    ui.label(
                                                        RichText::new(" ERR ")
                                                            .background_color(Color32::from_rgb(
                                                                120, 0, 0,
                                                            ))
                                                            .color(Color32::WHITE),
                                                    );

                                                    ui.with_layout(
                                                        egui::Layout::right_to_left(
                                                            egui::Align::Center,
                                                        ),
                                                        |ui| {
                                                            // Display the error string on the right side instead of the timestamp
                                                            ui.label(
                                                                RichText::new(error.name())
                                                                    .color(Color32::LIGHT_RED)
                                                            ).on_hover_text(error.to_string());

                                                            Self::widget_url(ui, url, false);
                                                        },
                                                    );
                                                });
                                            }
                                        });
                                });
                        }
                    });
            });
    }

    fn render_ui(&mut self, ui: &mut egui::Ui) {
        let menu_frame = egui::Frame::default()
            .fill(ui.visuals().extreme_bg_color)
            .inner_margin(egui::Margin::symmetric(8, 4));

        let graph_frame = egui::Frame::default()
            .fill(ui.visuals().extreme_bg_color)
            .inner_margin(egui::Margin::symmetric(0, 0));

        let panel_frame = egui::Frame::window(&ui.style());

        let left_panel_default_size = 220.0;
        let right_window_width = 200.0;
        let right_window_margin = 5.0;

        self.render_top_menu(ui, menu_frame);
        self.render_crawl_window(ui);
        self.render_about_window(ui);
        self.render_left_panel(ui, panel_frame, left_panel_default_size);

        let center_offset = right_window_width + right_window_margin;
        self.render_central_panel(ui, graph_frame, center_offset);

        let anchor_y_offset = 26.0;
        let minimap_reserved_height = 178.0;
        let cards_spacing = 4.0;
        self.render_right_window(
            ui,
            right_window_margin as i8,
            anchor_y_offset,
            right_window_width,
            minimap_reserved_height,
            cards_spacing,
        );
    }
}

impl eframe::App for ViewEgui {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll_channel_events();
        self.poll_graph_events();
        self.render_ui(ui);
    }
}
