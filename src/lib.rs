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

        /*
        let mut sim = Sim::new(12_000);
        let n = sim.real.len();

        for i in 0..n / 18 {
            sim.real[i] = 0.1;
            sim.imag[i] = 0.1;
        }
        */

        //sim.real[n / 2] = 3.;
        //sim.real[1] = 1.;

        let n = 1000;

        let mut wave1 = wave_packet(DT * (n as f32 / 100.), 0., 1., 1.5, n);
        let wave2 = wave_packet(DT * (n as f32 / 100.), 5., 1., 1.5, n);
        wave1.real.iter_mut().zip(&wave2.real).for_each(|(a, b)| *a += b);
        wave1.imag.iter_mut().zip(&wave2.imag).for_each(|(a, b)| *a += b);

        let sim = wave1;

        Self(sim)
    }
}

impl ClientState {
    fn update(&mut self, io: &mut EngineIo, _query: &mut QueryResult) {
        let Self(sim) = self;
        for _ in 0..100 {
            sim.step();
        }

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

            let plus_1 = (i + 1) % n;
            let minu_1 = i.checked_sub(1).unwrap_or(n - 1);

            self.imag[i] += c * (self.real[plus_1] + self.real[minu_1] + 2. * self.real[i]);
            self.real[i] -= c * (self.imag[plus_1] + self.imag[minu_1] + 2. * self.imag[i]);
        }
    }
}

fn sim_to_mesh(sim: &Sim) -> Mesh {
    let mut mesh = Mesh::new();

    let n = sim.real.len();
    for i in 0..n {
        let f = i as f32 / n as f32;
        let colors = [(&sim.real, [1., 0., 0.]), (&sim.imag, [0., 0.5, 1.])];
        for (part, color) in colors {
            let pos = [(f * TAU).cos(), part[i], (f * TAU).sin()];
            let idx = mesh.push_vertex(Vertex::new(pos, color));
            mesh.push_indices(&[idx]);
        }

        let mag = sim.real[i].powi(2) + sim.imag[i].powi(2);
        let pos = [(f * TAU).cos(), mag + 1., (f * TAU).sin()];
        let idx = mesh.push_vertex(Vertex::new(pos, [1.; 3]));
        mesh.push_indices(&[idx]);
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
