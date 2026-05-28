// src/gui.rs
use eframe;
use egui::{Color32, Pos2, RichText};
use egui_graphs::{FruchtermanReingoldWithCenterGravityState, Layout};
use petgraph::{
    Directed, Undirected,
    stable_graph::{DefaultIx, StableGraph},
};
use rand::RngExt;

///////////////////

#[derive(PartialEq)]
enum LeftTab {
    Activity,
    Queue,
    Errors,
}

//////////////////

pub struct EguiView {
    graph: egui_graphs::Graph<(), (), Undirected>,
    pub show_markdown_window: bool,
    pub show_outbound_window: bool,
    pub show_inbound_window: bool,
    pub markdown_text: String,
    //
    left_tab: LeftTab,
    selected_node: bool,
    auto_center: bool,
    // Card expansion states
    node_details_expanded: bool,
    hubs_expanded: bool,
    broken_expanded: bool,
}

impl EguiView {
    pub fn new() -> Self {
        let mut graph = Self::generate_basic_graph();
        Self::distribute_nodes_circle_generic(&mut graph);

        let text = "# Regnata removete motis patris

## Quid atque

Lorem markdownum finibus memorique ignis per alvo longeque dea eburnas serior,
eheu lupos ferocis raptatur altis bicorni Flentibus soror! Scilicet tollit.

> Ad Philammon viarum genitas nullosque cervicis legem; per simul simulantis
> ignisque solvo num; tellus sequitur: oro. Quibus adspexisse Ilios. Mentisque
> vitae Saturnia; quidem resque squamea contingere curvamine artesque!

## Fretum prohibet virtute una

Et caput idem, ullo est ventusve suique deos Horamque stultos; tuta. Nos
[cognoscere](#adversaque-pulchrior-membra-vultu) vult bellatricemque timidum me
Haemum locorum remotus an iubent segetis **erat clauditur**. Patriaeque murice
et o densis `memory_cluster_xhtml` admonuit mare carcere armentum effundite
sacrificos magnum.

## Inquit quae Me terra

Quae `soap_quicktime` cacumina temptamina in Symplegadas tenebrae, sanguine
iactatis iussit. Scire coeptaeque altius data umbra praerupta cinctum serpentis
tosti et dos parantem hinc et ambit si quaerit ab quos maternos.

- Ille omnia
- Uti aere et
- Aevo quae fluctus
- Aries latratus levare et membra curvataque fatigatum
- Auroque huic tanti cum modo quaerente creatis

## Adversaque Pulchrior membra vultu

Locorum tenebrasque fumant arguitur, tetigere in boumque coepit consequitur olim
magnus tu inquit ut draconem haurit ut. Natos ulla velut Faunine, rorantia
puppis indagine femori, te fuit et.

1. Primum quod
2. Viam tua patetis miratur saucius glomerataque mater
3. Felicissima iamque ungues manibus reseratis hic subito
4. Vulgatos mulcendaque ausum Emathiique culmina vitae";

        Self {
            graph: graph,
            show_markdown_window: false,
            show_outbound_window: false,
            show_inbound_window: false,
            markdown_text: String::from(text),
            left_tab: LeftTab::Activity,
            selected_node: false,
            auto_center: true,
            node_details_expanded: true,
            hubs_expanded: true,
            broken_expanded: true,
        }
    }

    pub fn run(&self) -> eframe::Result<()> {
        let mut options = eframe::NativeOptions::default();

        options.viewport = egui::ViewportBuilder::default()
            .with_resizable(true)
            .with_inner_size([1920.0 / 2.0, (1080.0 / 2.0)])
            .with_active(false);

        eframe::run_native(
            "egui_graphs_basic_demo",
            options,
            Box::new(|_context| Ok(Box::new(EguiView::new()))),
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

impl EguiView {
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

impl eframe::App for EguiView {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        let menu_frame = egui::Frame::default()
            .fill(ui.visuals().extreme_bg_color)
            .inner_margin(egui::Margin::symmetric(8, 4));

        let graph_frame = egui::Frame::default()
            .fill(ui.visuals().extreme_bg_color)
            .inner_margin(egui::Margin::symmetric(8, 4));

        // 2. Global Top Menu
        egui::Panel::top("top_menu_bar")
            .frame(menu_frame)
            .show_inside(ui, |ui| {
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
                });
            });

        let panel_frame = egui::Frame::window(&ui.style());

        let mut show_crawl_window = false;
        if show_crawl_window {
            egui::Window::new("Start Crawl")
                .open(&mut show_crawl_window)
                .collapsible(false)
                .resizable(false)
                .anchor(egui::Align2::CENTER_CENTER, [0.0, 0.0])
                .show(ui.ctx(), |ui| {
                    ui.label("Configure your crawling job here.");
                    ui.add_space(8.0);
                    if ui.button("Start New Session").clicked() {}
                    if ui.button("End Session (Test)").clicked() {}
                });
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
                                    ui.heading(RichText::new("47").color(Color32::LIGHT_BLUE));
                                    ui.label("crawled");
                                });
                            });
                            cols[1].group(|ui| {
                                ui.vertical_centered_justified(|ui| {
                                    ui.heading(RichText::new("29").color(Color32::YELLOW));
                                    ui.label("queued");
                                });
                            });
                        });

                        ui.add_space(4.0);

                        ui.columns(2, |cols| {
                            cols[0].group(|ui| {
                                ui.vertical_centered_justified(|ui| {
                                    ui.heading(RichText::new("3").color(Color32::LIGHT_RED));
                                    ui.label("errors");
                                });
                            });
                            cols[1].group(|ui| {
                                ui.vertical_centered_justified(|ui| {
                                    ui.heading("340ms");
                                    ui.label("avg");
                                });
                            });
                        });

                        ui.add_space(4.0);
                    });

                egui::CentralPanel::default()
                    .frame(egui::Frame::NONE)
                    .show_inside(ui, |ui| match self.left_tab {
                        LeftTab::Activity => {
                            egui::ScrollArea::vertical()
                                .auto_shrink([false, false])
                                .show(ui, |ui| {
                                    let history = vec![
                                        ("12:04", "200", "/blog/getting-..."),
                                        ("12:04", "200", "/docs/api/v2"),
                                        ("12:03", "404", "/legacy/old-api"),
                                        ("12:03", "301", "/home → /"),
                                        ("12:03", "200", "/pricing"),
                                    ];
                                    for (time, code, path) in history {
                                        ui.horizontal(|ui| {
                                            ui.label(RichText::new(time).color(Color32::DARK_GRAY));
                                            let code_color = if code.starts_with('2') {
                                                Color32::GREEN
                                            } else if code.starts_with('4') {
                                                Color32::RED
                                            } else {
                                                Color32::YELLOW
                                            };
                                            ui.label(RichText::new(code).color(code_color));
                                            ui.label(path);
                                        });
                                    }
                                });
                        }
                        LeftTab::Queue => {
                            ui.label("Queue content goes here...");
                        }
                        LeftTab::Errors => {
                            ui.label("Errors content goes here...");
                        }
                    });
            });

        let custom_panel_frame = egui::Frame::default()
            .fill(ui.visuals().extreme_bg_color)
            .inner_margin(egui::Margin::same(0));

        let cards_spacing = 4.0;
        let down_triangle_icon = "🔻";
        let left_triangle_icon = "◀";

        // 4. Right Panel: Crawling Inspector (Sequential Cards)
        egui::Panel::right("right_crawling_inspector")
            .frame(custom_panel_frame)
            .resizable(true)
            .default_size(280.0)
            .show_inside(ui, |ui| {
                // auto_shrink([false, false]) ensures the scroll area claims the full width
                // preventing horizontal jumping when scrollbars appear.
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {

                    let card_frame = egui::Frame::default()
                        .fill(ui.visuals().window_fill)
                        .stroke(ui.visuals().widgets.noninteractive.bg_stroke)
                        .inner_margin(egui::Margin::same(12))
                        .corner_radius(0.0);

                    // --- SECTION 1: NODE ---
                    card_frame.show(ui, |ui| {
                        ui.set_min_width(ui.available_width());

                        ui.horizontal(|ui| {
                            ui.heading("⏺ Node Details");
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let icon = if self.node_details_expanded { down_triangle_icon } else { left_triangle_icon };
                                if ui.add(egui::Button::new(icon).frame(false)).clicked() {
                                    self.node_details_expanded = !self.node_details_expanded;
                                }
                            });
                        });

                        if self.node_details_expanded {
                            ui.add_space(4.0);
                            ui.separator();
                            ui.add_space(4.0);

                            ui.label("🟢 /docs/api/v2");
                            ui.add_space(8.0);

                            egui::Grid::new("node_details_grid")
                                .num_columns(2)
                                .striped(true)
                                .spacing([40.0, 4.0])
                                .show(ui, |ui| {
                                    fn add_stretched_right_cell(
                                        ui: &mut egui::Ui,
                                        content: impl FnOnce(&mut egui::Ui),
                                    ) {
                                        ui.scope(|ui| {
                                            ui.set_min_width(ui.available_width());
                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                content,
                                            );
                                        });
                                    }

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

                                    ui.label("Load time");
                                    add_stretched_right_cell(ui, |ui| {
                                        ui.label("280ms");
                                    });
                                    ui.end_row();

                                    ui.label("Page size");
                                    add_stretched_right_cell(ui, |ui| {
                                        ui.label("62 KB");
                                    });
                                    ui.end_row();

                                    ui.label("Links in");
                                    add_stretched_right_cell(ui, |ui| {
                                        ui.label("5");
                                    });
                                    ui.end_row();

                                    ui.label("Links out");
                                    add_stretched_right_cell(ui, |ui| {
                                        ui.label("9");
                                    });
                                    ui.end_row();
                                });
                            ui.add_space(10.0);

                            ui.horizontal(|ui| {
                                if ui.button("📝 Markdown").clicked() {
                                    self.show_markdown_window = !self.show_markdown_window;
                                }
                                if ui.button("Outbound").clicked() {
                                    self.show_outbound_window = !self.show_outbound_window;
                                }
                                if ui.button("Inbound").clicked() {
                                    self.show_inbound_window = !self.show_inbound_window;
                                }
                            });

                            ui.add_space(10.0);
                            ui.separator();
                            ui.add_space(10.0);

                            ui.label(RichText::new("OUTBOUND").color(Color32::DARK_GRAY));
                            ui.add_space(4.0);
                            ui.label("/docs/api/v3");
                            ui.separator();
                            ui.label("/changelog");
                            ui.separator();
                            ui.label("/pricing");
                        }
                    });

                    ui.add_space(cards_spacing);

                    // --- SECTION 2: HUBS ---
                    card_frame.show(ui, |ui| {
                        ui.set_min_width(ui.available_width());

                        ui.horizontal(|ui| {
                            ui.heading("🌐 Hubs");
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let icon = if self.hubs_expanded { down_triangle_icon } else { left_triangle_icon };
                                if ui.add(egui::Button::new(icon).frame(false)).clicked() {
                                    self.hubs_expanded = !self.hubs_expanded;
                                }
                            });
                        });

                        if self.hubs_expanded {
                            ui.add_space(4.0);
                            ui.separator();
                            ui.add_space(4.0);

                            ui.label(RichText::new("BY IN-DEGREE").color(Color32::DARK_GRAY));
                            ui.add_space(10.0);

                            let hubs = vec![
                                ("/docs", 12.0, 12.0),
                                ("/pricing", 9.0, 12.0),
                                ("/blog", 7.0, 12.0),
                                ("/about", 5.0, 12.0),
                                ("/contact", 3.0, 12.0),
                            ];

                            for (i, (path, value, max)) in hubs.iter().enumerate() {
                                ui.horizontal(|ui| {
                                    ui.label(format!("{}", i + 1));
                                    ui.vertical(|ui| {
                                        ui.horizontal(|ui| {
                                            ui.label(*path);
                                            ui.with_layout(
                                                egui::Layout::right_to_left(egui::Align::Center),
                                                |ui| {
                                                    ui.label(
                                                        RichText::new(format!("{} in", value))
                                                            .color(Color32::GRAY),
                                                    );
                                                },
                                            );
                                        });
                                        let progress = value / max;
                                        ui.add(
                                            egui::ProgressBar::new(progress)
                                                .desired_height(3.0)
                                                .fill(Color32::from_rgb(80, 200, 150)),
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
                            ui.heading("❌ Broken Links");
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                let icon = if self.broken_expanded { down_triangle_icon } else { left_triangle_icon };
                                if ui.add(egui::Button::new(icon).frame(false)).clicked() {
                                    self.broken_expanded = !self.broken_expanded;
                                }
                            });
                        });

                        if self.broken_expanded {
                            ui.add_space(4.0);
                            ui.separator();
                            ui.add_space(4.0);

                            ui.label("404 - /legacy/old-api");
                        }
                    });

                    ui.add_space(cards_spacing);
                });
            });

        // 5. Central Panel LAST
        egui::CentralPanel::default()
            .frame(graph_frame)
            .show_inside(ui, |ui| {
                let central_rect = ui.max_rect();

                if self.show_markdown_window {
                    egui::Window::new("Markdown Source")
                        .open(&mut self.show_markdown_window)
                        .resizable(true)
                        .collapsible(true)
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

                if self.show_inbound_window {
                    egui::Window::new("Inbound Links")
                        .open(&mut self.show_inbound_window)
                        .resizable(true)
                        .constrain_to(central_rect)
                        .default_size([300.0, 200.0])
                        .show(ui.ctx(), |ui| {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                ui.label("/home");
                                ui.label("/docs/index");
                                ui.label("/blog/release-notes");
                                ui.label("/sitemap.xml");
                                ui.label("/contact");
                            });
                        });
                }

                if self.show_outbound_window {
                    egui::Window::new("Outbound Links")
                        .open(&mut self.show_outbound_window)
                        .resizable(true)
                        .constrain_to(central_rect)
                        .default_size([300.0, 200.0])
                        .show(ui.ctx(), |ui| {
                            egui::ScrollArea::vertical().show(ui, |ui| {
                                ui.label("/docs/api/v3");
                                ui.label("/changelog");
                                ui.label("/pricing");
                                ui.label("https://github.com/example/repo");
                            });
                        });
                }

                let settings_navigation = &egui_graphs::SettingsNavigation::new()
                    .with_zoom_and_pan_enabled(true)
                    .with_fit_to_screen_enabled(false);

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

                ui.add(&mut view);

                /////////////////// MINIMAP
                egui::Window::new("Minimap Overlay")
                    .anchor(egui::Align2::RIGHT_BOTTOM, [-16.0, -16.0])
                    .resizable(false)
                    .collapsible(false)
                    .constrain_to(central_rect)
                    .title_bar(false)
                    .frame(egui::Frame::window(&ui.style()).inner_margin(8.0))
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
}
