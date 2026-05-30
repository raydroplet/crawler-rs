// src/gui.rs
use crate::crawler::{CrawlCommand, CrawlResponse, CrawlRequest};
use crossbeam_channel as crossbeam;
use eframe;
use egui::{Color32, Pos2, RichText};
use egui_graphs::{FruchtermanReingoldWithCenterGravityState, Layout};
use petgraph::{
    Directed, Undirected,
    stable_graph::{DefaultIx, StableGraph},
};
use rand::RngExt;
use reqwest::Url;

///////////////////

#[derive(PartialEq)]
enum LeftTab {
    Activity,
    Queue,
    Errors,
}

//////////////////

pub struct ViewEgui {
    graph: egui_graphs::Graph<(), (), Undirected>,
    pub show_markdown_window: bool,
    pub show_outbound_window: bool,
    pub markdown_text: String,
    //
    left_tab: LeftTab,
    free_graph_movement: bool,
    // Card expansion states
    node_details_expanded: bool,
    hubs_expanded: bool,
    broken_expanded: bool,
    graph_expanded: bool,
    //
    show_crawl_window: bool,
    crawl_input_url: String,
    crawl_input_depth: i32,
    //
    crawler_rx: flume::Receiver<CrawlResponse>,
    crawler_tx: flume::Sender<CrawlCommand>,
}

impl ViewEgui {
    pub fn new(
        app_response_rx: flume::Receiver<CrawlResponse>,
        app_command_tx: flume::Sender<CrawlCommand>,
    ) -> Self {
        let mut graph = Self::generate_basic_graph();
        Self::distribute_nodes_circle_generic(&mut graph);

        let text = "# Regnata removete motis patris

Lorem markdownum finibus memorique ignis per alvo longeque dea eburnas serior,
eheu lupos ferocis raptatur altis bicorni Flentibus soror! Scilicet tollit.

> Ad Philammon viarum genitas nullosque cervicis legem; per simul simulantis
> ignisque solvo num; tellus sequitur: oro. Quibus adspexisse Ilios. Mentisque";

        Self {
            graph: graph,
            show_markdown_window: false,
            show_outbound_window: false,
            markdown_text: String::from(text),
            left_tab: LeftTab::Activity,
            free_graph_movement: false,
            node_details_expanded: true,
            hubs_expanded: true,
            broken_expanded: true,
            graph_expanded: true,
            //
            show_crawl_window: false,
            crawl_input_url: String::from("https://"),
            crawl_input_depth: 2,
            //
            crawler_rx: app_response_rx,
            crawler_tx: app_command_tx,
        }
    }

    pub fn run(view: ViewEgui) -> eframe::Result<()> {
        let mut options = eframe::NativeOptions::default();

        options.viewport = egui::ViewportBuilder::default()
            .with_resizable(false)
            .with_inner_size([1920.0 / 2.0, (1080.0 / 2.0)])
            .with_active(false);

        eframe::run_native(
            "egui_graphs_basic_demo",
            options,
            Box::new(|_context| Ok(Box::new(view))),
        )
    }

    pub fn distribute_nodes_circle_generic<Ty: petgraph::EdgeType>(
        g: &mut egui_graphs::Graph<(), (), Ty, petgraph::stable_graph::DefaultIx>,
    ) {
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
    #[allow(dead_code)]
    fn generate_basic_petgraph() -> StableGraph<(), (), Undirected, DefaultIx> {
        let mut g: StableGraph<(), (), Undirected> = StableGraph::default();

        let a = g.add_node(());
        let b = g.add_node(());
        let c = g.add_node(());

        g.add_edge(a, b, ());
        g.add_edge(b, c, ());
        g.add_edge(c, a, ());

        g
    }

    fn generate_basic_graph() -> egui_graphs::Graph<(), (), Undirected, DefaultIx> {
        let petgraph: StableGraph<(), (), Undirected, DefaultIx> =
            Self::generate_random_petgraph(100, 300);
        let graph: egui_graphs::Graph<(), (), Undirected, DefaultIx> =
            egui_graphs::Graph::from(&petgraph);
        graph
    }

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

    #[allow(dead_code)]
    fn graph_types(&mut self) {
        type L1 = egui_graphs::LayoutHierarchical;
        type S1 = egui_graphs::LayoutStateHierarchical;
        let _view = egui_graphs::GraphView::<_, _, _, _, _, _, S1, L1>::new(&mut self.graph);

        type L2 =
            egui_graphs::LayoutForceDirected<egui_graphs::FruchtermanReingoldWithCenterGravity>;
        type S2 = egui_graphs::FruchtermanReingoldWithCenterGravityState;
        let _view = egui_graphs::GraphView::<_, _, _, _, _, _, S2, L2>::new(&mut self.graph);

        type L3 = egui_graphs::LayoutForceDirected<egui_graphs::FruchtermanReingold>;
        type S3 = egui_graphs::FruchtermanReingoldState;
        let _view = egui_graphs::GraphView::<_, _, _, _, _, _, S3, L3>::new(&mut self.graph);
    }
}

impl eframe::App for ViewEgui {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let menu_frame = egui::Frame::default()
            .fill(ui.visuals().extreme_bg_color)
            .inner_margin(egui::Margin::symmetric(8, 4));

        let graph_frame = egui::Frame::default()
            .fill(ui.visuals().extreme_bg_color)
            .inner_margin(egui::Margin::symmetric(0, 0));

        // 2. Global Top Menu
        if true {
            // TODO: assert if this is centered correctly
            egui::Panel::top("top_menu_bar")
                .frame(menu_frame)
                .show_inside(ui, |ui| {
                    // 1. Get the total width available in the panel
                    let available_width = ui.available_width();

                    // 2. Estimate the width of your menu items.
                    // You may need to tweak this number based on your font size and labels.
                    // "File" + "View" + "Graph" ≈ 150px
                    let estimated_menu_width = 150.0;

                    // 3. Calculate the padding needed on the left to center it
                    let left_padding = (available_width - estimated_menu_width) / 2.0;

                    ui.horizontal(|ui| {
                        // 4. Push the menu bar to the right by adding empty space
                        if left_padding > 0.0 {
                            ui.add_space(left_padding);
                        }

                        // 5. Draw the menu bar
                        egui::MenuBar::new().ui(ui, |ui| {
                            ui.menu_button("File", |ui| {
                                if ui.button("📂 Open Graph...").clicked() {
                                    println!("Open");
                                }
                                if ui.button("💾 Save State").clicked() {
                                    println!("Save");
                                }
                                ui.separator();
                                if ui.button("❌ Exit").clicked() {
                                    ui.send_viewport_cmd(egui::ViewportCommand::Close);
                                }
                            });

                            ui.menu_button("View", |ui| {
                                if ui.button("🔄 Reset Layout").clicked() {
                                    println!("Reset");
                                }
                                if ui.button("⚙ Settings").clicked() {
                                    println!("Settings");
                                }
                            });

                            ui.menu_button("Graph", |ui| {
                                if ui.button("🔄 Center").clicked() {
                                    self.free_graph_movement = !self.free_graph_movement;
                                    println!("Center");
                                }
                                if ui.button("🔄 Reorganize").clicked() {
                                    Self::distribute_nodes_circle_generic(&mut self.graph);
                                    println!("Reorganize");
                                }
                            });
                            if ui.button("Crawl").clicked() {
                                self.show_crawl_window = true;
                            };
                            if ui.button("About").clicked() {
                                // TODO:
                            };
                        });
                    });
                });
        }

        let panel_frame = egui::Frame::window(&ui.style());
        let mut close_window = false;

        if self.show_crawl_window {
            egui::Window::new("Start Crawl")
                .open(&mut self.show_crawl_window) // Borrows the variable here
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui.ctx(), |ui| {
                    ui.label("Configure your crawling job here.");
                    ui.add_space(8.0);

                    egui::Grid::new("crawl_input_grid")
                        .num_columns(2)
                        .spacing([10.0, 10.0])
                        .show(ui, |ui| {
                            ui.label("Root URL:");
                            ui.text_edit_singleline(&mut self.crawl_input_url);
                            ui.end_row();

                            ui.label("Depth:");
                            ui.add(egui::DragValue::new(&mut self.crawl_input_depth).range(0..=10));
                            ui.end_row();
                        });

                    ui.add_space(12.0);

                    ui.horizontal(|ui| {
                        if ui.button("▶ Start Session").clicked() {

                            let Ok(url) = Url::parse(&self.crawl_input_url) else {
                                println!("failed to parse"); // TODO: provide a better warning
                                return;
                            };

                            let request = CrawlRequest { 
                                source: url,
                                depth: self.crawl_input_depth,
                            };
                            let _ = self.crawler_tx.send(CrawlCommand::RequestCrawl(request));

                            // close_window = true;
                        }

                        if ui.button("⏹ End Session").clicked() {
                            let _ = self.crawler_tx.send(CrawlCommand::Terminate);

                            // close_window = true;
                        }
                    });
                });
        }

        // 3. Safely update the struct field now that the borrow is gone
        if close_window {
            self.show_crawl_window = false;
        }

        // 3. Left Panel
        egui::Panel::left("left_crawling_input")
            .frame(panel_frame.clone())
            .resizable(true)
            .default_size(220.0)
            .show_inside(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.selectable_value(&mut self.left_tab, LeftTab::Activity, "📈 Activity");
                    ui.selectable_value(&mut self.left_tab, LeftTab::Queue, "⏳ Queue");
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
                                        RichText::new("47"), /* .color(Color32::LIGHT_BLUE) */
                                    );
                                    ui.label(RichText::new("Crawled").weak());
                                });
                            });
                            cols[1].group(|ui| {
                                ui.vertical_centered_justified(|ui| {
                                    ui.heading(
                                        RichText::new("29"), /* .color(Color32::YELLOW) */
                                    );
                                    ui.label(RichText::new("Queued").weak());
                                });
                            });
                        });

                        ui.add_space(4.0);

                        ui.columns(2, |cols| {
                            cols[0].group(|ui| {
                                ui.vertical_centered_justified(|ui| {
                                    ui.heading(
                                        RichText::new("3"), /* .color(Color32::LIGHT_RED) */
                                    );
                                    ui.label(RichText::new("Errors").weak());
                                });
                            });
                            cols[1].group(|ui| {
                                ui.vertical_centered_justified(|ui| {
                                    ui.heading("340ms");
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
                                            let history = vec![
                                                (
                                                    "12:04",
                                                    "200",
                                                    "wikipedia.com",
                                                    "/blog/getting-started-with-graphing-tool",
                                                ),
                                                ("12:05", "200", "wikipedia.com", "/docs/api"),
                                                ("12:06", "301", "wikipedia.com", "/home"),
                                                ("12:07", "404", "wikipedia.com", "/broken-link"),
                                            ];

                                            for (time, code, domain, path) in history {
                                                ui.horizontal(|ui| {
                                                    let code_color = if code.starts_with("2") {
                                                        Color32::from_rgb(0, 100, 0)
                                                    } else if code.starts_with("4") {
                                                        Color32::from_rgb(100, 0, 0)
                                                    } else {
                                                        Color32::from_rgb(100, 100, 0)
                                                    };

                                                    // 1. Claim space on the left (Badge)
                                                    ui.label(
                                                        RichText::new(format!(" {} ", code))
                                                            .background_color(code_color)
                                                            .color(Color32::WHITE),
                                                    );

                                                    // 2. Anchor the rest of the layout to the right
                                                    ui.with_layout(
                                                        egui::Layout::right_to_left(
                                                            egui::Align::Center,
                                                        ),
                                                        |ui| {
                                                            // 3. Claim space on the right (Timestamp) FIRST
                                                            ui.label(
                                                                RichText::new(time)
                                                                    .color(Color32::DARK_GRAY),
                                                            );

                                                            // 4. Fill the remaining middle space with the truncated address
                                                            ui.with_layout(
                                                                egui::Layout::left_to_right(
                                                                    egui::Align::Center,
                                                                ),
                                                                |ui| {
                                                                    ui.spacing_mut()
                                                                        .item_spacing
                                                                        .x = 0.0;
                                                                    ui.label(domain);
                                                                    let weak_color = ui
                                                                        .visuals()
                                                                        .weak_text_color();
                                                                    ui.visuals_mut()
                                                                        .override_text_color =
                                                                        Some(weak_color);
                                                                    ui.add(
                                                                        egui::Label::new(path)
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

                        LeftTab::Queue => {
                            egui::Frame::NONE
                                .fill(ui.visuals().extreme_bg_color)
                                .inner_margin(egui::Margin::same(4))
                                .show(ui, |ui| {
                                    egui::ScrollArea::vertical()
                                        .auto_shrink([false, false])
                                        .show(ui, |ui| {
                                            // Queue data: (Depth, domain, path)
                                            let queue_items = vec![
                                        ("0", "wikipedia.com", "/entry-node"),
                                        ("1", "wikipedia.com", "/first-hop-link"),
                                        (
                                            "2",
                                            "wikipedia.com",
                                            "/deep-link-that-is-very-long-to-test-truncation",
                                        ),
                                    ];

                                            for (depth, domain, path) in queue_items {
                                                ui.horizontal(|ui| {
                                                    // Depth Badge (Blue)
                                                    ui.label(
                                                        RichText::new(format!(" D{} ", depth))
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
                                                                    ui.label(domain);
                                                                    let weak_color = ui
                                                                        .visuals()
                                                                        .weak_text_color();
                                                                    ui.visuals_mut()
                                                                        .override_text_color =
                                                                        Some(weak_color);
                                                                    ui.add(
                                                                        egui::Label::new(path)
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
                                            // Error data: (domain, path, error_string)
                                            let errors = vec![
                                                ("wikipedia.com", "/some-broken-page", "Timeout"),
                                                ("bad-domain.com", "/", "DNS Res"),
                                                (
                                                    "wikipedia.com",
                                                    "/another-long-path-that-fails",
                                                    "Refused",
                                                ),
                                            ];

                                            for (domain, path, err_str) in errors {
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
                                                                RichText::new(err_str)
                                                                    .color(Color32::LIGHT_RED),
                                                            );

                                                            ui.with_layout(
                                                                egui::Layout::left_to_right(
                                                                    egui::Align::Center,
                                                                ),
                                                                |ui| {
                                                                    ui.spacing_mut()
                                                                        .item_spacing
                                                                        .x = 0.0;
                                                                    ui.label(domain);
                                                                    let weak_color = ui
                                                                        .visuals()
                                                                        .weak_text_color();
                                                                    ui.visuals_mut()
                                                                        .override_text_color =
                                                                        Some(weak_color);
                                                                    ui.add(
                                                                        egui::Label::new(path)
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
                    });
            });

        let cards_spacing = 4.0;
        let down_triangle_icon = "🔻";
        let left_triangle_icon = "◀";

        fn add_stretched_right_cell(ui: &mut egui::Ui, content: impl FnOnce(&mut egui::Ui)) {
            ui.scope(|ui| {
                ui.set_min_width(ui.available_width());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), content);
            });
        }

        // 1. Calculate the available space and subtract the right window's width
        let right_window_width = 200.0;
        let right_margin = 5.0;
        // 150px (minimap) + 8px (inner frame margin) + 8px (bottom anchor margin) + 12px (visual gap)
        let minimap_reserved_height = 178.0;

        // 5. Central Panel
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
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                ui.add(
                                    egui::TextEdit::multiline(&mut self.markdown_text)
                                        .font(egui::TextStyle::Monospace)
                                        .code_editor()
                                        .interactive(false)
                                        .desired_width(f32::INFINITY),
                                );
                            });
                        });
                }

                if self.show_outbound_window {
                    let inspect_node = |id: u8| {};
                    let node_button = |ui: &mut egui::Ui, label: &str, id: u8| {
                        if ui.button(label).clicked() {
                            println!("{}", id);
                        }
                    };

                    egui::Window::new("Outbound Links")
                        .open(&mut self.show_outbound_window)
                        .resizable(true)
                        .collapsible(false)
                        .constrain_to(central_rect)
                        .default_size([300.0, 200.0])
                        .show(ui.ctx(), |ui| {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                node_button(ui, "/docs/api/v3", 0);
                                node_button(ui, "/changelog", 2);
                                node_button(ui, "/pricing", 3);
                                node_button(ui, "https://github.com/example/repo", 4);
                            });
                        });
                }

                ////////
                let settings_navigation = &egui_graphs::SettingsNavigation::new()
                    .with_zoom_and_pan_enabled(self.free_graph_movement)
                    .with_fit_to_screen_enabled(!self.free_graph_movement);

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
                .with_navigations(settings_navigation);

                // 1. Trick the center calculation by expanding the LEFT boundary off-screen.
                let mut virtual_rect = ui.available_rect_before_wrap();
                virtual_rect.min.x -= right_window_width + right_margin;

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

        // .frame(custom_panel_frame)
        // .resizable(false)
        // .default_size(200.0)
        // .show_separator_line(false)
        // .show_inside(ui, |ui| {

        let y_offset = 26.0;
        let margin_offset = right_margin as i8; // same value used for graph calculations
        let margin = egui::Margin {
            left: margin_offset,
            right: margin_offset,
            top: margin_offset,
            bottom: margin_offset,
        };
        egui::Window::new("Right_Inspector_Window")
            // Anchor to top right, with a 10px margin off the edges
            .anchor(egui::Align2::RIGHT_TOP, [0.0, y_offset]) // WARN: y to match the top bar height
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
                ui.set_width(right_window_width); // Force the width

                // Calculate the maximum allowed height so it doesn't overlap the minimap
                let max_height = ui.ctx().content_rect().height()
                    - y_offset
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

                                // 1. URL HEADER: High contrast, zero gap, and text wrapping
                                egui::Frame::none()
                                    .fill(ui.visuals().faint_bg_color) // Uses the active theme's subtle background color
                                    .rounding(6.0) // Rounds the corners
                                    .inner_margin(8.0) // Adds padding inside the background box
                                    .show(ui, |ui| {
                                        // Force the frame to stretch to the edges so the text centers nicely within the whole panel width
                                        ui.set_min_width(ui.available_width());

                                        ui.vertical_centered(|ui| {
                                            ui.label(
                                                egui::RichText::new("wikipedia.com")
                                                    .strong()
                                                    .size(12.0),
                                            );

                                            if true {
                                                // Mute the path so the domain stands out as the primary identifier
                                                ui.label(
                                                    egui::RichText::new("/post-post")
                                                        .color(ui.visuals().weak_text_color())
                                                        .size(10.0),
                                                );
                                            }
                                        });
                                    });

                                ui.add_space(8.0);

                                egui::Grid::new("node_details_grid")
                                    .num_columns(2)
                                    .striped(true)
                                    .spacing([40.0, 4.0])
                                    .show(ui, |ui| {
                                        ui.label("Status");
                                        add_stretched_right_cell(ui, |ui| {
                                            ui.label(
                                                RichText::new(" 200 OK ")
                                                    .background_color(Color32::from_rgb(0, 100, 0))
                                                    .color(Color32::WHITE),
                                            );
                                        });
                                        ui.end_row();

                                        ui.label("Depth");
                                        add_stretched_right_cell(ui, |ui| {
                                            ui.label("2");
                                        });
                                        ui.end_row();

                                        ui.label("Links out");
                                        add_stretched_right_cell(ui, |ui| {
                                            ui.label("9");
                                        });
                                        ui.end_row();

                                        ui.label("Page size");
                                        add_stretched_right_cell(ui, |ui| {
                                            ui.label("62 KB");
                                        });
                                        ui.end_row();

                                        ui.label("Load time");
                                        add_stretched_right_cell(ui, |ui| {
                                            ui.label("280ms");
                                        });
                                        ui.end_row();
                                    });

                                ui.add_space(8.0);

                                ui.horizontal(|ui| {
                                    if ui.button("📝 Markdown").clicked() {
                                        self.show_markdown_window = !self.show_markdown_window;
                                    }
                                    if ui.button("🕸 Outbound").clicked() {
                                        self.show_outbound_window = !self.show_outbound_window;
                                    }
                                });
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
                                egui::Grid::new("graph_details_grid")
                                    .num_columns(2)
                                    .striped(true)
                                    .spacing([40.0, 4.0])
                                    .show(ui, |ui| {
                                        ui.label("Nodes");
                                        add_stretched_right_cell(ui, |ui| {
                                            ui.label("2384");
                                        });
                                        ui.end_row();
                                        //
                                        ui.label("Links");
                                        add_stretched_right_cell(ui, |ui| {
                                            ui.label("3002");
                                        });
                                        ui.end_row();
                                        //
                                    });

                                ui.add_space(8.0);

                                ui.horizontal(|ui| {
                                    // NOTE: icons needed
                                    if ui.button("@ Center").clicked() {
                                        self.free_graph_movement = !self.free_graph_movement;
                                    }
                                    if ui.button("@ Reorganize").clicked() {
                                        Self::distribute_nodes_circle_generic(&mut self.graph);
                                    }
                                });
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

                            if self.hubs_expanded {
                                ui.add_space(4.0);
                                ui.separator();
                                ui.add_space(4.0);

                                let max = 12.0;
                                let hubs = vec![
                                    ("wikipedia.com", 12.0),
                                    ("reddit.com", 10.0),
                                    ("rust-lang.org", 9.0),
                                ];

                                for (path, value) in hubs.iter() {
                                    ui.horizontal(|ui| {
                                        ui.vertical(|ui| {
                                            ui.horizontal(|ui| {
                                                ui.label(*path);
                                                ui.with_layout(
                                                    egui::Layout::right_to_left(
                                                        egui::Align::Center,
                                                    ),
                                                    |ui| {
                                                        ui.label(
                                                            RichText::new(format!("{}", value))
                                                                .color(Color32::GRAY),
                                                        );
                                                    },
                                                );
                                            });
                                            let progress = value / max;
                                            ui.add(
                                                egui::ProgressBar::new(progress)
                                                    .desired_height(3.0)
                                                    .fill(Color32::from_rgb(40, 100, 180)),
                                            );
                                        });
                                    });
                                    ui.add_space(4.0);
                                }
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

                            let broken = vec![
                                ("broken-website.web", 404),
                                ("123.com", 401),
                                ("square.circle", 418),
                            ];
                            if self.broken_expanded {
                                ui.add_space(4.0);
                                ui.separator();
                                ui.add_space(4.0);

                                for (address, code) in broken.iter() {
                                    ui.horizontal(|ui| {
                                        ui.label(*address);
                                        add_stretched_right_cell(ui, |ui| {
                                            ui.label(
                                                RichText::new(format!(" {} ", code))
                                                    .background_color(Color32::from_rgb(80, 10, 10))
                                                    .color(Color32::WHITE),
                                            );
                                        });
                                    });
                                }
                            }
                        });
                    });
            });
    }
}
