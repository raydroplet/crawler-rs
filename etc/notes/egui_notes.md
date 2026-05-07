```rust
// NOTE:
egui_graphs/src/with_extras, defines specializations for:
FruchtermanReingoldWithCenterGravity && FruchtermanReingoldWithCenterGravityState


// NOTE:
// (executor/plain-data)?
pub type FruchtermanReingoldWithCenterGravity = FruchtermanReingoldWithExtras<(Extra<CenterGravity, true>, ())>;
pub type FruchtermanReingoldWithCenterGravityState = FruchtermanReingoldWithExtrasState<(Extra<CenterGravity, true>, ())>;


// NOTE: reads current UI layout
pub fn from_ui_fr_state(ui: &mut egui::Ui) -> egui_graphs::LayoutSpec {
    let st = egui_graphs::get_layout_state::<
        egui_graphs::FruchtermanReingoldWithCenterGravityState,
    >(ui, None);
    LayoutSpec::FruchtermanReingold {
        running: Some(st.base.is_running),
        dt: Some(st.base.dt),
        epsilon: Some(st.base.epsilon),
        damping: Some(st.base.damping),
        max_step: Some(st.base.max_step),
        k_scale: Some(st.base.k_scale),
        c_attract: Some(st.base.c_attract),
        c_repulse: Some(st.base.c_repulse),
        extras: Some(vec![ExtrasSpec::CenterGravity {
            enabled: Some(st.extras.0.enabled),
            c: Some(st.extras.0.params.c),
        }]),
    }
}


// NOTE: random graph
// (Graph -> GraphView)?
let value: egui_graphs::Graph = egui_graphs::generate_random_graph(10, 10);


// NOTE: this can help shape a random graph
Self::distribute_nodes_circle_generic(&mut g);


// NOTE: graph widget declaration
(DemoGraph::Directed(ref mut g), DemoLayout::FruchtermanReingold) => {
    if let Some(spec::PendingLayout::FR(st)) = self.pending_layout.take() {
        egui_graphs::set_layout_state::<FruchtermanReingoldWithCenterGravityState>(
            ui, st, None,
        );
    }
    let mut view = egui_graphs::GraphView::<
        _,
        _,
        _,
        _,
        _,
        _,
        FruchtermanReingoldWithCenterGravityState,
        LayoutForceDirected<FruchtermanReingoldWithCenterGravity>,
    >::new(g)
    .with_interactions(settings_interaction)
    .with_navigations(settings_navigation)
    .with_styles(settings_style);
    #[cfg(feature = "events")]
    {
        #[cfg(not(target_arch = "wasm32"))]
        {
            view = view.with_event_sink(&self.event_publisher);
        }
        #[cfg(target_arch = "wasm32")]
        {
            view = view.with_event_sink(&self.events_buf);
        }
    }
    ui.add(&mut view);
}

// NOTE: egui_graphs::GraphView parameters
() — Node Weight: The data attached to the nodes.
() — Edge Weight: The data attached to the edges.
petgraph::Directed (or Undirected) — Edge Type: Whether the graph has arrows.
petgraph::stable_graph::DefaultIx — Index Type: The integer size used for IDs (usually u32).
egui_graphs::DefaultNodeShape — Node Shape: How the node is drawn.
egui_graphs::DefaultEdgeShape — Edge Shape: How the edge is drawn.
FruchtermanReingoldWithCenterGravityState — Layout State: The runtime parameters of the layout.
LayoutForceDirected<...> — Layout Algorithm: The physics engine used.

// TODO:
// implement a custom Layout to handle varying node size and correct node repulsion/collisions

// NOTE:
// The Graph may be dumb data container, and the GraphView may be the active engine that performs the calculations and updates that container.
```
