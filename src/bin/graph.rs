use crossbeam_channel::{Receiver, Sender, unbounded};
use eframe::{App, CreationContext, NativeOptions, run_native};
use egui::Pos2;
use egui_graphs::{
    FruchtermanReingoldWithCenterGravity, FruchtermanReingoldWithCenterGravityState, Graph,
    GraphView, LayoutForceDirected,
};
use fdg_sim::{ForceGraph, ForceGraphHelper, Simulation, SimulationParameters};
use petgraph::stable_graph::NodeIndex as StableNodeIndex;
use std::thread;
use std::time::Duration;

type L = LayoutForceDirected<FruchtermanReingoldWithCenterGravity>;
type S = FruchtermanReingoldWithCenterGravityState;

enum AppCommand {
    SetRunning(bool),
    Expand(usize),
}

struct BenchmarkApp {
    egui_graph: Graph<(), ()>,
    nodes: Vec<StableNodeIndex>,
    edges_count: usize,
    nodes_per_tick: usize,
    is_running: bool,
    free_graph_movement: bool,

    // Thread communication channels
    tx_cmd: Sender<AppCommand>,
    rx_pos: Receiver<Vec<Pos2>>,
}

impl BenchmarkApp {
    fn new(cc: &CreationContext<'_>) -> Self {
        let (tx_cmd, rx_cmd) = unbounded::<AppCommand>();
        let (tx_pos, rx_pos) = unbounded::<Vec<Pos2>>();
        let ctx_clone = cc.egui_ctx.clone();

        let mut egui_graph = Graph::new(petgraph::stable_graph::StableGraph::new());
        let mut nodes = Vec::new();
        let mut edges_count = 0;

        // 1. Initialize UI Graph Topology
        for i in 0..25 {
            let egui_idx = egui_graph.add_node(());
            let angle = i as f32 * 2.4;
            let radius = 15.0 + (i as f32 * 2.0);
            let pos_egui = Pos2::new(angle.cos() * radius, angle.sin() * radius);

            if let Some(node) = egui_graph.node_mut(egui_idx) {
                node.set_location(pos_egui);
            }

            if i > 0 {
                let target_egui = nodes[i % nodes.len()];
                egui_graph.add_edge(target_egui, egui_idx, ());
                edges_count += 1;
            }
            nodes.push(egui_idx);
        }

        // 2. Spawn Isolated Physics Thread
        thread::spawn(move || {
            let mut fdg_graph: ForceGraph<(), ()> = ForceGraph::default();
            let mut all_fdg_nodes = Vec::new();

            // Mirror initial topology deterministically
            for i in 0..25 {
                let fdg_idx = fdg_graph.add_force_node("", ());
                let angle = i as f32 * 2.4;
                let radius = 15.0 + (i as f32 * 2.0);
                fdg_graph[fdg_idx].location.x = angle.cos() * radius;
                fdg_graph[fdg_idx].location.y = angle.sin() * radius;
                fdg_graph[fdg_idx].location.z = 0.0;

                if i > 0 {
                    let target_fdg = all_fdg_nodes[i % all_fdg_nodes.len()];
                    fdg_graph.add_edge(target_fdg, fdg_idx, ());
                }
                all_fdg_nodes.push(fdg_idx);
            }

            let mut fdg_sim = Simulation::from_graph(fdg_graph, SimulationParameters::default());
            let mut is_running = false;

            // Closure infers fdg_sim's older petgraph node type automatically
            let mut process_cmd = |sim: &mut Simulation<(), ()>,
                                   nodes: &mut Vec<_>,
                                   running: &mut bool,
                                   cmd: AppCommand| {
                match cmd {
                    AppCommand::SetRunning(r) => *running = r,
                    AppCommand::Expand(count) => {
                        let start_idx = nodes.len();
                        for i in 0..count {
                            let current_idx = start_idx + i;
                            let fdg_idx = sim.get_graph_mut().add_force_node("", ());

                            let angle = current_idx as f32 * 2.4;
                            let radius = 15.0 + (current_idx as f32 * 0.5);
                            sim.get_graph_mut()[fdg_idx].location.x = angle.cos() * radius;
                            sim.get_graph_mut()[fdg_idx].location.y = angle.sin() * radius;
                            sim.get_graph_mut()[fdg_idx].location.z = 0.0;

                            if !nodes.is_empty() {
                                let prng = current_idx.wrapping_mul(1103515245).wrapping_add(12345);
                                let target_idx = prng % nodes.len();
                                let target_fdg = nodes[target_idx];
                                sim.get_graph_mut().add_edge(fdg_idx, target_fdg, ());
                            }
                            nodes.push(fdg_idx);
                        }
                    }
                }
            };

            loop {
                if is_running {
                    while let Ok(cmd) = rx_cmd.try_recv() {
                        process_cmd(&mut fdg_sim, &mut all_fdg_nodes, &mut is_running, cmd);
                    }

                    if is_running {
                        fdg_sim.update(0.016);

                        let positions: Vec<Pos2> = all_fdg_nodes
                            .iter()
                            .map(|&idx| {
                                let loc = &fdg_sim.get_graph()[idx].location;
                                Pos2::new(loc.x, loc.y)
                            })
                            .collect();

                        let _ = tx_pos.send(positions);

                        ctx_clone.request_repaint();
                        thread::sleep(Duration::from_millis(16));
                    }
                } else {
                    if let Ok(cmd) = rx_cmd.recv() {
                        process_cmd(&mut fdg_sim, &mut all_fdg_nodes, &mut is_running, cmd);
                    } else {
                        break;
                    }
                }
            }
        });

        Self {
            egui_graph,
            nodes,
            edges_count,
            nodes_per_tick: 100,
            is_running: false,
            free_graph_movement: false,
            tx_cmd,
            rx_pos,
        }
    }

    fn inject_nodes(&mut self) {
        let start_idx = self.nodes.len();

        for i in 0..self.nodes_per_tick {
            let current_idx = start_idx + i;
            let egui_idx = self.egui_graph.add_node(());

            let angle = current_idx as f32 * 2.4;
            let radius = 15.0 + (current_idx as f32 * 0.5);
            let pos_egui = Pos2::new(angle.cos() * radius, angle.sin() * radius);

            if let Some(node) = self.egui_graph.node_mut(egui_idx) {
                node.set_location(pos_egui);
            }

            if !self.nodes.is_empty() {
                let prng = current_idx.wrapping_mul(1103515245).wrapping_add(12345);
                let target_idx = prng % self.nodes.len();
                let target_egui = self.nodes[target_idx];

                self.egui_graph.add_edge(egui_idx, target_egui, ());
                self.edges_count += 1;
            }
            self.nodes.push(egui_idx);
        }

        let _ = self.tx_cmd.send(AppCommand::Expand(self.nodes_per_tick));
    }

    fn sync_positions(&mut self) {
        let mut latest_frame = None;

        // Drain the channel instantly to grab only the most recent calculation
        while let Ok(positions) = self.rx_pos.try_recv() {
            latest_frame = Some(positions);
        }

        if let Some(pos_vec) = latest_frame {
            // Zip safely binds the two arrays, preventing index out-of-bounds panics
            // during the split-second delay when the UI has expanded but physics hasn't.
            for (&egui_idx, &pos) in self.nodes.iter().zip(pos_vec.iter()) {
                if let Some(egui_node) = self.egui_graph.node_mut(egui_idx) {
                    egui_node.set_location(pos);
                }
            }
        }
    }
}

fn add_stretched_right_cell(ui: &mut egui::Ui, add_contents: impl FnOnce(&mut egui::Ui)) {
    ui.with_layout(
        egui::Layout::right_to_left(egui::Align::Center),
        add_contents,
    );
}

impl App for BenchmarkApp {
    fn logic(&mut self, _ctx: &egui::Context, _frame: &mut eframe::Frame) {}

    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.sync_positions();

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
                        self.is_running = !self.is_running;
                        let _ = self.tx_cmd.send(AppCommand::SetRunning(self.is_running));
                    }
                    if row_ui.button("🎯 Center").clicked() {
                        self.free_graph_movement = !self.free_graph_movement;
                    }
                });
            });

        // Graph Rendering
        egui::CentralPanel::default().show_inside(ui, |central_ui| {
            let mut state = egui_graphs::get_layout_state::<S>(central_ui, None);
            state.base.is_running = false;

            let settings_navigation = &egui_graphs::SettingsNavigation::new()
                .with_zoom_and_pan_enabled(self.free_graph_movement)
                .with_fit_to_screen_enabled(!self.free_graph_movement);

            egui_graphs::set_layout_state::<S>(central_ui, state, None);

            let mut view = GraphView::<_, _, _, _, _, _, S, L>::new(&mut self.egui_graph)
                .with_navigations(settings_navigation);

            central_ui.add(&mut view);
        });
    }
}

fn main() -> eframe::Result<()> {
    run_native(
        "fdg_sim Native Benchmark (Multi-threaded)",
        NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(BenchmarkApp::new(cc)))),
    )
}
