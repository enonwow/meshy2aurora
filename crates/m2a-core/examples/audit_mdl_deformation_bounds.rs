use std::{env, fs, path::PathBuf, process::ExitCode};

use m2a_core::mdl::{evaluate_skin_deformation_v1, inspect_binary_mdl};
use serde_json::json;

fn main() -> ExitCode {
    match run() {
        Ok(report) => {
            println!("{report}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn run() -> Result<String, String> {
    let path = env::args()
        .nth(1)
        .map(PathBuf::from)
        .ok_or_else(|| "usage: audit_mdl_deformation_bounds <model.mdl>".to_owned())?;
    let bytes = fs::read(&path).map_err(|error| format!("model read failed: {error}"))?;
    let report = inspect_binary_mdl(&bytes).map_err(|error| error.to_string())?;
    let mut samples = Vec::new();
    let mut global = Bounds::new();
    let mut bind = Bounds::new();

    for animation in &report.animations {
        let mut times = vec![0.0, animation.length * 0.5, animation.length];
        times.dedup_by(|left, right| left.to_bits() == right.to_bits());
        for time in times {
            let sample = evaluate_skin_deformation_v1(&report, &animation.name, time)
                .map_err(|error| format!("{} at {time}: {}", animation.name, error.message))?;
            let mut sampled_bounds = Bounds::new();
            let mut bind_bounds = Bounds::new();
            for skin in &sample.skins {
                for vertex in &skin.vertices {
                    sampled_bounds.include(vertex.sampled_world);
                    bind_bounds.include(vertex.bind_world);
                    global.include(vertex.sampled_world);
                    bind.include(vertex.bind_world);
                }
            }
            samples.push(json!({
                "clipName": animation.name,
                "timeSeconds": time,
                "sampledBounds": sampled_bounds.as_json(),
                "bindBounds": bind_bounds.as_json(),
                "maxAbsoluteSampledCoordinate": sampled_bounds.max_absolute_coordinate()
            }));
        }
    }

    serde_json::to_string_pretty(&json!({
        "schemaVersion": 1,
        "modelPath": path,
        "modelName": report.model.name,
        "animationCount": report.animations.len(),
        "sampleCount": samples.len(),
        "globalSampledBounds": global.as_json(),
        "globalBindBounds": bind.as_json(),
        "maxAbsoluteSampledCoordinate": global.max_absolute_coordinate(),
        "samples": samples
    }))
    .map_err(|error| error.to_string())
}

struct Bounds {
    min: [f32; 3],
    max: [f32; 3],
}

impl Bounds {
    fn new() -> Self {
        Self {
            min: [f32::INFINITY; 3],
            max: [f32::NEG_INFINITY; 3],
        }
    }

    fn include(&mut self, point: [f32; 3]) {
        for axis in 0..3 {
            self.min[axis] = self.min[axis].min(point[axis]);
            self.max[axis] = self.max[axis].max(point[axis]);
        }
    }

    fn max_absolute_coordinate(&self) -> f32 {
        self.min
            .iter()
            .chain(&self.max)
            .map(|value| value.abs())
            .fold(0.0, f32::max)
    }

    fn as_json(&self) -> serde_json::Value {
        json!({
            "min": self.min,
            "max": self.max
        })
    }
}
