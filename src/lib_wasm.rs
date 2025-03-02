//! WebAssembly bindings for Turmoil
//!
//! This module contains the WebAssembly bindings for Turmoil, making the library
//! more accessible from JavaScript/TypeScript.

use js_sys::{Function as JsFunction, Promise};
use std::str::FromStr;
use wasm_bindgen::prelude::*;
use web_time::Duration;

use crate::{net::IpAddr, Builder, Sim};

#[wasm_bindgen(js_name = "Host")]
pub struct WasmHost {
    ip: IpAddr,
    sim: WasmSim,
}

#[wasm_bindgen(js_class = "Host")]
impl WasmHost {
    #[wasm_bindgen]
    pub fn new(ip: &str, sim: WasmSim) -> Result<WasmHost, JsValue> {
        let ip_addr = match IpAddr::from_str(ip) {
            Ok(addr) => addr,
            Err(e) => return Err(JsValue::from_str(&format!("Invalid IP address: {}", e))),
        };

        Ok(WasmHost { ip: ip_addr, sim })
    }

    #[wasm_bindgen]
    pub fn ip(&self) -> String {
        self.ip.to_string()
    }

    #[wasm_bindgen]
    pub fn is_running(&mut self) -> bool {
        self.sim.0.is_host_running(self.ip)
    }

    #[wasm_bindgen]
    pub fn crash(&mut self) {
        self.sim.0.crash(self.ip);
    }

    #[wasm_bindgen]
    pub fn bounce(&mut self) {
        self.sim.0.bounce(self.ip);
    }
}

/// A handle for interacting with the simulation.
#[wasm_bindgen(js_name = "Sim")]
pub struct WasmSim(pub(crate) Sim<'static>);

#[wasm_bindgen(js_class = "Sim")]
impl WasmSim {
    #[wasm_bindgen]
    /// How much logical time has elapsed since the simulation started.
    pub fn elapsed(&self) -> f64 {
        self.0.elapsed().as_millis() as f64
    }

    #[wasm_bindgen]
    /// How many steps have been taken since the simulation started.
    pub fn steps(&self) -> usize {
        self.0.steps
    }

    #[wasm_bindgen]
    /// Run the simulation for the configured duration.
    pub fn run(&mut self) -> Result<(), JsValue> {
        match self.0.run() {
            Ok(_) => Ok(()),
            Err(e) => Err(JsValue::from_str(&format!("Simulation failed: {}", e))),
        }
    }

    #[wasm_bindgen]
    /// Register a host with the simulation using a JavaScript callback.
    /// Returns a WasmHost object that can be used to interact with the host.
    pub fn host(&mut self, name: &str, callback: JsFunction) -> Result<(), JsValue> {
        // Properly use the callback in a way that allows the simulation to execute it
        self.0.host(name, move || {
            let callback = callback.clone();
            
            // Return a future that will be executed in the simulation
            async move {
                // Create a new promise each time this function is called
                let promise = Promise::new(&mut |resolve, reject| {
                    let this = JsValue::null();
                    match callback.call0(&this) {
                        Ok(result) => {
                            // Just resolve with the result
                            let _ = resolve.call1(&JsValue::null(), &result);
                        }
                        Err(e) => {
                            let _ = reject.call1(
                                &JsValue::null(),
                                &JsValue::from_str(&format!("Host callback error: {:?}", e)),
                            );
                        }
                    }
                });

                // Wait for the promise to resolve or reject
                match wasm_bindgen_futures::JsFuture::from(promise).await {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        let err = std::io::Error::new(
                            std::io::ErrorKind::Other,
                            format!("Host callback failed: {:?}", e),
                        );
                        Err(Box::new(err) as Box<dyn std::error::Error>)
                    }
                }
            }
        });

        Ok(())
    }

    #[wasm_bindgen]
    /// Register a client with the simulation using a JavaScript callback.
    pub fn client(&mut self, name: &str, callback: JsFunction) -> Result<(), JsValue> {
        // Clone the callback so we can move it into the async block
        let callback = callback.clone();
        
        // Properly use the callback in a way that allows the simulation to execute it
        self.0.client(name, async move {
            // Create a new promise each time this function is called
            let promise = Promise::new(&mut |resolve, reject| {
                let this = JsValue::null();
                match callback.call0(&this) {
                    Ok(result) => {
                        // Just resolve with the result
                        let _ = resolve.call1(&JsValue::null(), &result);
                    }
                    Err(e) => {
                        let _ = reject.call1(
                            &JsValue::null(),
                            &JsValue::from_str(&format!("Client callback error: {:?}", e)),
                        );
                    }
                }
            });

            // Wait for the promise to resolve or reject
            match wasm_bindgen_futures::JsFuture::from(promise).await {
                Ok(_) => Ok(()),
                Err(e) => {
                    let err = std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Client callback failed: {:?}", e),
                    );
                    Err(Box::new(err) as Box<dyn std::error::Error>)
                }
            }
        });

        Ok(())
    }

    #[wasm_bindgen]
    /// Partition the network between two hosts or sets of hosts.
    pub fn partition(&mut self, a: &str, b: &str) -> Result<(), JsValue> {
        self.0.partition(a, b);
        Ok(())
    }

    #[wasm_bindgen]
    /// Repair the network between two hosts or sets of hosts.
    pub fn repair(&mut self, a: &str, b: &str) -> Result<(), JsValue> {
        self.0.repair(a, b);
        Ok(())
    }

    #[wasm_bindgen]
    /// Hold messages between two hosts or sets of hosts.
    pub fn hold(&mut self, a: &str, b: &str) -> Result<(), JsValue> {
        self.0.hold(a, b);
        Ok(())
    }

    #[wasm_bindgen]
    /// Release held messages between two hosts or sets of hosts.
    pub fn release(&mut self, a: &str, b: &str) -> Result<(), JsValue> {
        self.0.release(a, b);
        Ok(())
    }

    #[wasm_bindgen]
    /// Crash the hosts that match the pattern.
    pub fn crash(&mut self, pattern: &str) -> Result<(), JsValue> {
        self.0.crash(pattern);
        Ok(())
    }

    #[wasm_bindgen]
    /// Bounce (restart) the hosts that match the pattern.
    pub fn bounce(&mut self, pattern: &str) -> Result<(), JsValue> {
        self.0.bounce(pattern);
        Ok(())
    }

    #[wasm_bindgen]
    /// Run the simulation step by step for specified number of steps.
    pub fn step(&mut self, steps: usize) -> Result<(), JsValue> {
        for _ in 0..steps {
            match self.0.step() {
                Ok(_) => {}
                Err(e) => return Err(JsValue::from_str(&format!("Step failed: {}", e))),
            }
        }
        Ok(())
    }

    #[wasm_bindgen]
    /// Get the status of the simulation as a JSON string.
    pub fn status(&self) -> String {
        format!(
            "{{\"elapsed\": {}, \"steps\": {}}}",
            self.elapsed(),
            self.steps()
        )
    }
}

/// Export Builder with additional browser-friendly methods
#[wasm_bindgen(js_name = "Builder")]
pub struct TurmoilBuilder(Builder);

#[wasm_bindgen(js_class = "Builder")]
impl TurmoilBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        TurmoilBuilder(Builder::new())
    }

    pub fn build(&self) -> WasmSim {
        WasmSim(self.0.build())
    }

    pub fn simulation_duration_ms(&mut self, duration_ms: f64) -> Self {
        self.0
            .simulation_duration(Duration::from_millis(duration_ms as u64));
        Self(self.0.clone())
    }

    pub fn tick_duration_ms(&mut self, duration_ms: f64) -> Self {
        self.0
            .tick_duration(Duration::from_millis(duration_ms as u64));
        Self(self.0.clone())
    }

    pub fn fail_rate(&mut self, rate: f64) -> Self {
        self.0.fail_rate(rate);
        Self(self.0.clone())
    }

    pub fn repair_rate(&mut self, rate: f64) -> Self {
        self.0.repair_rate(rate);
        Self(self.0.clone())
    }

    pub fn min_message_latency_ms(&mut self, latency_ms: f64) -> Self {
        self.0
            .min_message_latency(Duration::from_millis(latency_ms as u64));
        Self(self.0.clone())
    }

    pub fn max_message_latency_ms(&mut self, latency_ms: f64) -> Self {
        self.0
            .max_message_latency(Duration::from_millis(latency_ms as u64));
        Self(self.0.clone())
    }
}
