use std::f32::consts::TAU;

use cimvr_common::{
    render::{Mesh, MeshHandle, Primitive, Render, UploadMesh, Vertex},
    Transform,
};
use cimvr_engine_interface::{make_app_state, pkg_namespace, prelude::*, println};

mod array2d;

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

        sched.add_system(Self::update).build();

        let mut sim = Sim::new(1000);
        let n = sim.real.len();
        //sim.real[n/2] = 1.;
        sim.real[1] = 1.;

        //let mut sim = wave_packet(DT * 50., 0., 1., 0.1, 1001);

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

const HBAR: f32 = 1.;
const M: f32 = 1.;
const DT: f32 = 0.1;
const DX: f32 = 1.;

impl Sim {
    pub fn new(n: usize) -> Self {
        Self {
            imag: vec![0.; n],
            real: vec![0.; n],
        }
    }

    pub fn step(&mut self) {
        let n = self.real.len();

        for i in 0..n {
            let c = HBAR * DT / (2. * M * DX.powi(2));

            self.imag[i] += c * (self.real[(i + 1) % n] + self.real[i.checked_sub(1).unwrap_or(n-1)] + 2. * self.real[i]);
            self.real[i] -= c * (self.imag[(i + 1) % n] + self.imag[i.checked_sub(1).unwrap_or(n-1)] + 2. * self.imag[i]);
        }
    }
}

fn sim_to_mesh(sim: &Sim) -> Mesh {
    let mut mesh = Mesh::new();

    let n = sim.real.len();
    for i in 0..n {
        let f = i as f32 / n as f32;
        for (part, color) in [(&sim.real, [1., 0., 0.]), (&sim.imag, [0., 0.2, 1.])] {
            let pos = [(f*TAU).cos(), part[i], (f*TAU).sin()];
            let idx = mesh.push_vertex(Vertex::new(pos, color));
            mesh.push_indices(&[idx]);
        }
    }

    mesh
}

// Defines entry points for the engine to hook into.
// Calls new() for the appropriate state.
make_app_state!(ClientState, DummyUserState);

fn wave_packet(sigma: f32, x0: f32, k: f32, amp: f32, n: usize) -> Sim {
    let (real, imag) = (0..n)
        .map(|i| {
            let x = ((i as f32 / n as f32) * 2. - 1.) * DT * n as f32 / 2.;

            let c = amp * ((-(x - x0).powi(2)) / (2. * sigma.powi(2))).exp();
            let v = k * x;
            (c * v.cos(), c * v.sin())
        })
        .unzip();

    Sim { real, imag }
}
