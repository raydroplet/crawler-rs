use eframe;
use egui::Pos2;
use egui_graphs::{FruchtermanReingoldWithCenterGravityState, Layout};
use petgraph::{
    Directed,
    Undirected,
    // graph::IndexType,
    stable_graph::{/* EdgeIndex, NodeIndex, */ DefaultIx, StableGraph},
};
use rand::RngExt;

pub struct EguiView {
    graph: egui_graphs::Graph<(), (), Undirected>,
    pub show_markdown_window: bool,
    pub markdown_text: String, // Store the raw markdown text here
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
            markdown_text: String::from(text), // Store the raw markdown text here
        }
    }

    pub fn run(&self) -> eframe::Result<()> {
        eframe::run_native(
            "egui_graphs_basic_demo",
            eframe::NativeOptions::default(),
            Box::new(|_context| Ok(Box::new(EguiView::new()))),
        )
    }

    // taken from egui_graphs source code
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
        // let petgraph: StableGraph<(), (), Undirected, DefaultIx> = Self::generate_basic_petgraph();
        let petgraph: StableGraph<(), (), Undirected, DefaultIx> =
            Self::generate_random_petgraph(100, 300);
        let graph: egui_graphs::Graph<(), (), Undirected, DefaultIx> =
            egui_graphs::Graph::from(&petgraph);
        //
        graph
    }

    /// Generates a random graph with the specified number of nodes and edges.
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
        // default random
        // let _view = egui_graphs::DefaultGraphView::new(&mut self.graph);

        // hierarchical
        type L1 = egui_graphs::LayoutHierarchical;
        type S1 = egui_graphs::LayoutStateHierarchical;
        let _view = egui_graphs::GraphView::<_, _, _, _, _, _, S1, L1>::new(&mut self.graph);

        // Force‑Directed (FR) with Center Gravity
        type L2 =
            egui_graphs::LayoutForceDirected<egui_graphs::FruchtermanReingoldWithCenterGravity>;
        type S2 = egui_graphs::FruchtermanReingoldWithCenterGravityState;
        let _view = egui_graphs::GraphView::<_, _, _, _, _, _, S2, L2>::new(&mut self.graph);

        // in-depth force directed layout
        type L3 = egui_graphs::LayoutForceDirected<egui_graphs::FruchtermanReingold>;
        type S3 = egui_graphs::FruchtermanReingoldState;
        let _view = egui_graphs::GraphView::<_, _, _, _, _, _, S3, L3>::new(&mut self.graph);

        // extra: composable addons
        // type L = egui_graphs::LayoutForceDirected<egui_graphs::FruchtermanReingoldWithCenterGravity>;
        // type S = egui_graphs::FruchtermanReingoldWithCenterGravityState;
        // let mut state = egui_graphs::GraphView::<_, _, _, _, _, _, S, L>::get_layout_state(ui);
        // state.base.is_running = true;
        // state.extras.0.params.c = 0.2;
        // egui_graphs::graph_view::set_layout_state(ui, state);
        // let _view = egui_graphs::GraphView::<_, _, _, _, _, _, S, L>::new(&mut self.graph);
    }
}

impl eframe::App for EguiView {
    fn ui(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
        // 1. Define the shared frame WITH proper margins
        let menu_frame = egui::Frame::default()
            .fill(ui.visuals().extreme_bg_color)
            // Add 8px padding on the left/right, and 4px on the top/bottom.
            // This gives the buttons breathing room without breaking the edge-to-edge background.
            .inner_margin(egui::Margin::symmetric(8, 4));

        let graph_frame = egui::Frame::default()
            .fill(ui.visuals().extreme_bg_color)
            // Add 8px padding on the left/right, and 4px on the top/bottom.
            // This gives the buttons breathing room without breaking the edge-to-edge background.
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

        // 3. Left Panel: Crawling Input (Styled)
        egui::Panel::left("left_crawling_input")
            .frame(panel_frame.clone()) // Apply the window style
            .resizable(true)
            .default_size(220.0)
            .show_inside(ui, |ui| {
                ui.add_space(4.0);
                ui.heading("Crawling Input");
                ui.separator();
                ui.label("Seed URL:");
                ui.add_space(10.0);
                ui.label("Max Depth: 3");
                ui.label("Concurrency: 10");
            });

        // 4. Right Panel: Crawling Inspector (Styled)
        egui::Panel::right("right_crawling_inspector")
            .frame(panel_frame) // Apply the window style
            .resizable(true)
            .default_size(280.0)
            .show_inside(ui, |ui| {
                ui.add_space(4.0);
                ui.heading("Inspector");
                ui.separator();
                ui.label("Selected Node Details:");
                ui.add_space(4.0);
                ui.label("No node selected.");

                if ui.button("📝 Open Markdown Source").clicked() {
                    self.show_markdown_window = !self.show_markdown_window;
                }
            });

        // 5. Central Panel LAST
        egui::CentralPanel::default()
            .frame(graph_frame)
            .show_inside(ui, |ui| {
                // // --- Inner Toolbar (Centered) ---
                // egui::Panel::top("top_graph_bar")
                //     .frame(menu_frame) // Inherits the 4px vertical breathing room
                //     .show_inside(ui, |ui| {
                //
                //         // Use a horizontal layout and calculate the exact center
                //         ui.horizontal(|ui| {
                //             // 3 standard buttons are roughly 180 pixels wide total.
                //             // We find the middle of the screen, and subtract half of that width (90.0)
                //             let center_offset = (ui.available_width() / 2.0) - 90.0;
                //
                //             // Push the buttons to the middle
                //             ui.add_space(center_offset.max(0.0));
                //
                //             if ui.button("▶ Play").clicked() {}
                //             if ui.button("⏸ Pause").clicked() {}
                //             if ui.button("⏹ Stop").clicked() {}
                //         });
                //
                //     });

                // 1. Get the exact bounding box of the Central Panel
                let central_rect = ui.max_rect();

                // 2. Draw the floating window, but constrain it to that box
                if self.show_markdown_window {
                    egui::Window::new("Markdown Source")
                        .open(&mut self.show_markdown_window)
                        .resizable(true)
                        .collapsible(true)
                        // --- THIS IS THE MAGIC LINE ---
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
                // ui.separator();

                // --- The Graph Widget ---
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
                >::new(&mut self.graph);

                ui.add(&mut view);
            });
    }
}

// impl EguiView {
//     fn panels(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
//         // panel for menu bar
//         let mut style = egui::Frame::new().inner_margin(4);
//         let mut cmd = Command::Nothing;
//
//         egui::Panel::top("wrap_app_top_bar")
//             .frame(style)
//             .show_inside(ui, |ui| {
//                 ui.horizontal_wrapped(|ui| {
//                     ui.visuals_mut().button_frame = false;
//                     self.bar_contents(ui, frame, &mut cmd);
//                 });
//             });
//     }
//
//     fn bar_contents(&mut self, ui: &mut egui::Ui, frame: &mut eframe::Frame, cmd: &mut Command) {
//         ui.add_space(8.0);
//         ui.separator();
//         ui.menu_button("💻 Backend", |ui| {
//             ui.set_style(ui.global_style()); // ignore the "menu" style set by `menu_button`.
//             self.backend_panel_contents(ui, frame, cmd);
//         });
//
//         ui.separator();
//
//         let mut selected_anchor = self.state.selected_anchor;
//         for (name, anchor, _app) in self.apps_iter_mut() {
//             if ui
//                 .selectable_label(selected_anchor == anchor, name)
//                 .clicked()
//             {
//                 selected_anchor = anchor;
//                 if frame.is_web() {
//                     ui.open_url(egui::OpenUrl::same_tab(format!("#{anchor}")));
//                 }
//             }
//         }
//         self.state.selected_anchor = selected_anchor;
//
//         ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
//             if false {
//                 // TODO(emilk): fix the overlap on small screens
//                 if clock_button(ui, crate::seconds_since_midnight()).clicked() {
//                     self.state.selected_anchor = Anchor::Clock;
//                     if frame.is_web() {
//                         ui.open_url(egui::OpenUrl::same_tab("#clock"));
//                     }
//                 }
//             }
//
//             egui::warn_if_debug_build(ui);
//         });
//     }
// }
//
