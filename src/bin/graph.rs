use eframe::{App, CreationContext, NativeOptions, run_native};
use egui::{Pos2, Color32, Shape, Stroke};
use egui_graphs::{
    FruchtermanReingoldWithCenterGravity, FruchtermanReingoldWithCenterGravityState, Graph,
    LayoutForceDirected, DisplayEdge, EdgeProps, Node, DisplayNode, DrawContext, DefaultNodeShape
};
use petgraph::stable_graph::{NodeIndex, StableGraph, IndexType};
use petgraph::{EdgeType, Undirected};

type _L = LayoutForceDirected<FruchtermanReingoldWithCenterGravity>;
type _S = FruchtermanReingoldWithCenterGravityState;

////////////////////
/// custom drawing

// 1. Define a stateless unit struct
#[derive(Clone)]
pub struct FixedWidthEdgeShape;

// 2. Implement From<EdgeProps<E>> (Only E is generic here in 0.30.0)
impl<E: Clone> From<EdgeProps<E>> for FixedWidthEdgeShape {
    fn from(_props: EdgeProps<E>) -> Self {
        Self // No internal state needed
    }
}

// 3. Implement the DisplayEdge trait
impl<N: Clone, E: Clone, Ty: EdgeType, Ix: IndexType, Dn: DisplayNode<N, E, Ty, Ix>>
    DisplayEdge<N, E, Ty, Ix, Dn> for FixedWidthEdgeShape
{
fn shapes(
        &mut self,
        start: &Node<N, E, Ty, Ix, Dn>,
        end: &Node<N, E, Ty, Ix, Dn>,
        ctx: &DrawContext<'_>,
    ) -> Vec<Shape> {
        // 1. Transform world (canvas) coordinates into UI screen coordinates
        let start_pos = ctx.meta.canvas_to_screen_pos(start.location());
        let end_pos = ctx.meta.canvas_to_screen_pos(end.location());

        // 2. Since we are in pure screen space, a width of 2.0 is naturally
        // fixed to 2 physical pixels on the monitor, ignoring zoom.
        let stroke = Stroke::new(1.0, Color32::GRAY);

        vec![Shape::line_segment(
            [start_pos, end_pos],
            stroke,
        )]
    }

    fn update(&mut self, _state: &EdgeProps<E>) {
        // Nothing to update since we don't store state
    }

    fn is_inside(
        &self,
        _start: &Node<N, E, Ty, Ix, Dn>,
        _end: &Node<N, E, Ty, Ix, Dn>,
        _pos: Pos2,
    ) -> bool {
        false
    }
}

type FixedWidthGraph = egui_graphs::Graph<(), (), Undirected, u32, DefaultNodeShape, FixedWidthEdgeShape>;

///
////////////////////
struct GraphState {
    is_running: bool,
    _show_advanced: bool,
    delta: f32,
    damping: f32,
    max_step: f32,
    epsilon: f32,
    k_scale: f32,
    c_attract: f32,
    c_repulse: f32,
    has_center_gravity: bool,
    center_strength: f32,
}

struct BenchmarkApp {
    egui_graph: FixedWidthGraph, // Replaced Graph<(), ()>
    nodes: Vec<NodeIndex>,
    edges_count: usize,
    nodes_per_tick: usize,
    graph_state: GraphState,
    free_graph_movement: bool,
}

impl BenchmarkApp {
    fn new(_cc: &CreationContext<'_>) -> Self {
        // let mut egui_graph = Graph::new(StableGraph::new());
        let mut egui_graph = FixedWidthGraph::new(StableGraph::default());
        let mut nodes = Vec::new();
        let mut edges_count = 0;

        for i in 0..5 {
            let idx = egui_graph.add_node(());

            let angle = i as f32 * 2.4;
            let radius = 15.0 + (i as f32 * 2.0);
            let pos = Pos2::new(angle.cos() * radius, angle.sin() * radius);

            if let Some(node) = egui_graph.node_mut(idx) {
                node.set_location(pos);
            }

            if i > 0 {
                let target = nodes[i % nodes.len()];
                egui_graph.add_edge(target, idx, ());
                edges_count += 1;
            }
            nodes.push(idx);
        }

        let state = GraphState {
            is_running: true,
            _show_advanced: false,
            delta: 0.100,
            damping: 0.01,
            max_step: 3.0,
            epsilon: 0.015,
            k_scale: 3.0,
            c_attract: 1.0,
            c_repulse: 1.0,
            has_center_gravity: true,
            center_strength: 0.30,
        };

        Self {
            egui_graph,
            nodes,
            edges_count,
            nodes_per_tick: 100,
            graph_state: state,
            free_graph_movement: false,
        }
    }

    fn inject_nodes(&mut self) {
        let start_idx = self.nodes.len();

        for i in 0..self.nodes_per_tick {
            let current_idx = start_idx + i;
            let new_node = self.egui_graph.add_node(());

            let angle = current_idx as f32 * 2.4;
            let radius = 15.0 + (current_idx as f32 * 0.5);
            let pos = Pos2::new(angle.cos() * radius, angle.sin() * radius);

            if let Some(node) = self.egui_graph.node_mut(new_node) {
                node.set_location(pos);
            }

            if !self.nodes.is_empty() {
                let prng = current_idx.wrapping_mul(1103515245).wrapping_add(12345);
                let target_idx = prng % self.nodes.len();

                let target = self.nodes[target_idx];
                self.egui_graph.add_edge(new_node, target, ());
                self.edges_count += 1;
            }
            self.nodes.push(new_node);
        }
    }
}

// Helper function to align text to the right side of the grid cell
fn add_stretched_right_cell(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    ui.with_layout(
        egui::Layout::right_to_left(egui::Align::Center),
        add_contents,
    );
}

impl App for BenchmarkApp {
    fn logic(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Timer removed. Interaction is now purely driven by the UI window.
    }

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        // Floating Information & Control Window
        egui::Window::new("Graph Details")
            .resizable(false)
            .default_width(0.0)
            .show(ui.ctx(), |window_ui| {
                egui::Grid::new("graph_details_grid")
                    .num_columns(2)
                    .striped(true)
                    .spacing([40.0, 4.0])
                    .show(window_ui, |grid_ui| {
                        grid_ui.label("Nodes");
                        add_stretched_right_cell(grid_ui, |cell_ui| {
                            cell_ui.label(self.nodes.len().to_string());
                        });
                        grid_ui.end_row();

                        grid_ui.label("Links");
                        add_stretched_right_cell(grid_ui, |cell_ui| {
                            cell_ui.label(self.edges_count.to_string());
                        });
                        grid_ui.end_row();
                    });

                window_ui.separator();

                window_ui.horizontal(|row_ui| {
                    if row_ui.button("Expand").clicked() {
                        self.inject_nodes();
                    }
                    if row_ui.button("Animated").clicked() {
                        self.graph_state.is_running = !self.graph_state.is_running;
                    }
                    if row_ui.button("🎯 Center").clicked() {
                        self.free_graph_movement = !self.free_graph_movement;
                    }
                });
            });

        // Graph Rendering
        egui::CentralPanel::default().show_inside(ui, |central_ui| {
            let mut state = egui_graphs::get_layout_state::<
                FruchtermanReingoldWithCenterGravityState,
            >(central_ui, None);

            state.base.is_running = self.graph_state.is_running;
            state.base.dt = self.graph_state.delta;
            state.base.damping = self.graph_state.damping;
            state.base.max_step = self.graph_state.max_step;
            state.base.epsilon = self.graph_state.epsilon;
            state.base.k_scale = self.graph_state.k_scale;
            state.base.c_attract = self.graph_state.c_attract;
            state.base.c_repulse = self.graph_state.c_repulse;
            state.extras.0.enabled = self.graph_state.has_center_gravity;
            state.extras.0.params.c = self.graph_state.center_strength;

            let settings_navigation = &egui_graphs::SettingsNavigation::new()
                .with_zoom_and_pan_enabled(self.free_graph_movement)
                .with_fit_to_screen_enabled(!self.free_graph_movement);

            egui_graphs::set_layout_state::<FruchtermanReingoldWithCenterGravityState>(
                central_ui, state, None,
            );

            let mut view = egui_graphs::GraphView::<
                _,
                _,
                _,
                _,
                _,
                _,
                FruchtermanReingoldWithCenterGravityState,
                egui_graphs::LayoutForceDirected<egui_graphs::FruchtermanReingoldWithCenterGravity>,
            >::new(&mut self.egui_graph)
            .with_navigations(settings_navigation);

            central_ui.add(&mut view);
        });

        ui.ctx().request_repaint();
    }
}

fn main() -> eframe::Result<()> {
    run_native(
        "Fruchterman-Reingold Native Benchmark",
        NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(BenchmarkApp::new(cc)))),
    )
}
