use eframe;
use egui_graphs::{FruchtermanReingoldWithCenterGravityState, Layout};
use petgraph::{
    // graph::IndexType,
    stable_graph::{/* EdgeIndex, NodeIndex, */ DefaultIx, StableGraph},
    Directed, Undirected
};
use rand::RngExt;
use egui::Pos2;

pub struct EguiView {
    graph: egui_graphs::Graph<(), (), Undirected>,
}

impl EguiView {
    pub fn new() -> Self {
        let mut graph = Self::generate_basic_graph();
        Self::distribute_nodes_circle_generic(&mut graph);

        Self {
            graph: graph,
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
        let petgraph: StableGraph<(), (), Undirected, DefaultIx> = Self::generate_random_petgraph(100, 300);
        let graph: egui_graphs::Graph<(), (), Undirected, DefaultIx> = egui_graphs::Graph::from(&petgraph);
        //
        graph
    }

    /// Generates a random graph with the specified number of nodes and edges.
    pub fn generate_random_petgraph(num_nodes: usize, num_edges: usize) -> StableGraph<(), (), Undirected, DefaultIx> {
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
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show_inside(ui, |ui| {
            let mut view = egui_graphs::GraphView::<
                _,
                _,
                _,
                _,
                _,
                _,
                FruchtermanReingoldWithCenterGravityState,
                egui_graphs::LayoutForceDirected<egui_graphs::FruchtermanReingoldWithCenterGravity>,
            >::new(&mut self.graph);

            ui.add(&mut view);
        });
    }
}

// NOTE:
// egui_graphs/src/with_extras, defines specializations for:
// FruchtermanReingoldWithCenterGravity && FruchtermanReingoldWithCenterGravityState

// NOTE:
// (executor/plain-data)?
//
//  pub type FruchtermanReingoldWithCenterGravity = FruchtermanReingoldWithExtras<(Extra<CenterGravity, true>, ())>;
//  pub type FruchtermanReingoldWithCenterGravityState = FruchtermanReingoldWithExtrasState<(Extra<CenterGravity, true>, ())>;

// NOTE: reads current UI layout
//
// pub fn from_ui_fr_state(ui: &mut egui::Ui) -> egui_graphs::LayoutSpec {
//     let st = egui_graphs::get_layout_state::<
//         egui_graphs::FruchtermanReingoldWithCenterGravityState,
//     >(ui, None);
//     LayoutSpec::FruchtermanReingold {
//         running: Some(st.base.is_running),
//         dt: Some(st.base.dt),
//         epsilon: Some(st.base.epsilon),
//         damping: Some(st.base.damping),
//         max_step: Some(st.base.max_step),
//         k_scale: Some(st.base.k_scale),
//         c_attract: Some(st.base.c_attract),
//         c_repulse: Some(st.base.c_repulse),
//         extras: Some(vec![ExtrasSpec::CenterGravity {
//             enabled: Some(st.extras.0.enabled),
//             c: Some(st.extras.0.params.c),
//         }]),
//     }
// }

// NOTE: random graph
//
// let value: egui_graphs::Graph = egui_graphs::generate_random_graph(10, 10);
//
// (Graph -> GraphView)?

// NOTE: this can help shape a random graph
//
// Self::distribute_nodes_circle_generic(&mut g);

// NOTE: graph widget declaration
//
// (DemoGraph::Directed(ref mut g), DemoLayout::FruchtermanReingold) => {
//     if let Some(spec::PendingLayout::FR(st)) = self.pending_layout.take() {
//         egui_graphs::set_layout_state::<FruchtermanReingoldWithCenterGravityState>(
//             ui, st, None,
//         );
//     }
//     let mut view = egui_graphs::GraphView::<
//         _,
//         _,
//         _,
//         _,
//         _,
//         _,
//         FruchtermanReingoldWithCenterGravityState,
//         LayoutForceDirected<FruchtermanReingoldWithCenterGravity>,
//     >::new(g)
//     .with_interactions(settings_interaction)
//     .with_navigations(settings_navigation)
//     .with_styles(settings_style);
//     #[cfg(feature = "events")]
//     {
//         #[cfg(not(target_arch = "wasm32"))]
//         {
//             view = view.with_event_sink(&self.event_publisher);
//         }
//         #[cfg(target_arch = "wasm32")]
//         {
//             view = view.with_event_sink(&self.events_buf);
//         }
//     }
//     ui.add(&mut view);
// }

// NOTE: egui_graphs::GraphView parameters
//
// () — Node Weight: The data attached to the nodes.
// () — Edge Weight: The data attached to the edges.
// petgraph::Directed (or Undirected) — Edge Type: Whether the graph has arrows.
// petgraph::stable_graph::DefaultIx — Index Type: The integer size used for IDs (usually u32).
// egui_graphs::DefaultNodeShape — Node Shape: How the node is drawn.
// egui_graphs::DefaultEdgeShape — Edge Shape: How the edge is drawn.
// FruchtermanReingoldWithCenterGravityState — Layout State: The runtime parameters of the layout.
// LayoutForceDirected<...> — Layout Algorithm: The physics engine used.

// TODO:
// implement a custom Layout to handle varying node size and correct node repulsion/collisions

// NOTE:
// The Graph may be dumb data container, and the GraphView may be the active engine that performs the calculations and updates that container.

