use egui_graphs;
use eframe;

pub struct EguiView {
    graph: egui_graphs::Graph,
}

impl EguiView {
    pub fn new() -> Self {
        let graph = Self::generate_graph();
        Self {
            graph: egui_graphs::Graph::from(&graph),
        }
    }

    pub fn run(&self) -> eframe::Result<()> {
        eframe::run_native(
            "egui_graphs_basic_demo",
            eframe::NativeOptions::default(),
            Box::new(|_context| Ok(Box::new(EguiView::new()))),
        )
    }

    fn generate_graph() -> petgraph::stable_graph::StableGraph<(), ()> {
        let mut g = petgraph::stable_graph::StableGraph::new();

        let a = g.add_node(());
        let b = g.add_node(());
        let c = g.add_node(());

        g.add_edge(a, b, ());
        g.add_edge(b, c, ());
        g.add_edge(c, a, ());

        g
    }
}

impl eframe::App for EguiView {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            type L = egui_graphs::LayoutHierarchical;
            type S = egui_graphs::LayoutStateHierarchical;
            let mut graph_view = egui_graphs::GraphView::<_,_,_,_,_,_,S,L>::new(&mut self.graph);

            ui.add(&mut graph_view);
        });
    }
}

//
