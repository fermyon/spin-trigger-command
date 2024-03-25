use std::sync::Arc;

use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use spin_app::AppComponent;
use spin_core::Engine;
use spin_trigger::{
    cli::NoArgs, EitherInstance, EitherInstancePre, TriggerAppEngine, TriggerExecutor,
};

pub(crate) type RuntimeData = ();

pub struct CommandTrigger {
    engine: TriggerAppEngine<Self>,
    components: Vec<Component>,
}

#[derive(Clone, Debug)]
pub struct Component {
    pub id: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TriggerMetadata {
    pub r#type: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CommandTriggerConfig {
    pub component: String,
}

#[async_trait]
impl TriggerExecutor for CommandTrigger {
    const TRIGGER_TYPE: &'static str = "command";
    type RuntimeData = RuntimeData;
    type TriggerConfig = CommandTriggerConfig;
    type RunConfig = NoArgs;

    async fn new(engine: TriggerAppEngine<Self>) -> Result<Self> {
        let components = engine
            .trigger_configs()
            .map(|(_, config)| Component {
                id: config.component.clone(),
            })
            .collect();
        Ok(Self { engine, components })
    }

    async fn run(self, _config: Self::RunConfig) -> Result<()> {
        let components = self.components;
        let engine = Arc::new(self.engine);

        // TODO:
        // This only uses first component for now.
        let executor = Executor::new(engine, components[0].clone());

        executor.run().await
    }

    // TODO:
    // This function assumes WASI P1
    async fn instantiate_pre(
        engine: &Engine<Self::RuntimeData>,
        component: &AppComponent,
        _config: &Self::TriggerConfig,
    ) -> Result<EitherInstancePre<Self::RuntimeData>> {
        let module = component.load_module(engine).await?;
        Ok(EitherInstancePre::Module(
            engine
                .module_instantiate_pre(&module)
                .with_context(|| format!("Failed to instantiate component '{}'", component.id()))?,
        ))
    }
}

pub struct Executor {
    pub engine: Arc<TriggerAppEngine<CommandTrigger>>,
    pub component: Component,
}

impl Executor {
    pub fn new(engine: Arc<TriggerAppEngine<CommandTrigger>>, component: Component) -> Self {
        Self { engine, component }
    }

    pub async fn run(&self) -> Result<()> {
        let store_builder = self
            .engine
            .store_builder(&self.component.id, spin_core::WasiVersion::Preview1)?;
        let (instance, mut store) = self
            .engine
            .prepare_instance_with_store(&self.component.id, store_builder)
            .await?;

        let EitherInstance::Module(instance) = instance else {
            unreachable!()
        };

        let start = instance
            .get_func(&mut store, "")
            .or_else(|| instance.get_func(&mut store, "_start"))
            .context("Expected component to export _start function")?;

        let _ = start.call_async(&mut store, &[], &mut []).await?;

        Ok(())
    }
}
