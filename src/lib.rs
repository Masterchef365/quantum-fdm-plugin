use cimvr_common::{render::{Mesh, Vertex, Render, MeshHandle, Primitive, UploadMesh}, Transform};
use cimvr_engine_interface::{make_app_state, prelude::*, println, pkg_namespace};

// All state associated with client-side behaviour
struct ClientState(Sim);

const WAVE_RDR: MeshHandle = MeshHandle::new(pkg_namespace!("Wave"));

impl UserState for ClientState {
    // Implement a constructor
    fn new(io: &mut EngineIo, sched: &mut EngineSchedule<Self>) -> Self {
        io.create_entity()
            .add_component(Transform::new())
            .add_component(Render::new(WAVE_RDR).primitive(Primitive::Points))
            .build();

        sched.add_system(Self::update)
            .build();

        let mut sim = Sim::new(100);

        Self(sim)
    }
}

impl ClientState {
    fn update(&mut self, io: &mut EngineIo, _query: &mut QueryResult) {
        let Self(sim) = self;
        sim.step();

        io.send(&UploadMesh {
            id: WAVE_RDR,
            mesh: sim_to_mesh(&sim),
        })
    }
}

struct Sim {
    imag: Vec<f32>,
    real: Vec<f32>,
}

impl Sim {
    pub fn new(n: usize) -> Self {
        Self {
            imag: vec![0.; n],
            real: vec![0.; n],
        }
    }

    pub fn step(&mut self) {}
}

fn sim_to_mesh(sim: &Sim) -> Mesh {
    let mut mesh = Mesh::new();

    let n = sim.real.len();
    for i in 0..n {
        let f = i as f32 / n as f32;
        for (part, color) in [(&sim.real, [1., 0., 0.]), (&sim.imag, [0., 0.2, 1.])] {
            let pos = [f, part[i], 0.];
            let idx = mesh.push_vertex(Vertex::new(pos, color));
            mesh.push_indices(&[idx]);
        }
    }

    mesh
}

// Defines entry points for the engine to hook into.
// Calls new() for the appropriate state.
make_app_state!(ClientState, DummyUserState);
