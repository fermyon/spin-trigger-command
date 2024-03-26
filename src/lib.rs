use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use spin_app::AppComponent;
use spin_core::Engine;
use spin_trigger::TriggerInstancePre;
use spin_trigger::{cli::NoArgs, TriggerAppEngine, TriggerExecutor};

pub(crate) type RuntimeData = ();
pub(crate) type Store = spin_core::Store<RuntimeData>;

pub struct CommandTrigger {
    engine: TriggerAppEngine<Self>,
    components: Vec<Component>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Component {
    pub id: String,
    #[serde(default)]
    pub executor: CommandExecutorType,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct TriggerMetadata {
    pub r#type: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "lowercase", tag = "type")]
pub enum CommandExecutorType {
    #[default]
    Preview2,
    Preview1,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CommandTriggerConfig {
    pub component: String,
    #[serde(default)]
    pub executor: CommandExecutorType,
}

pub enum CommandInstancePre {
    Component(spin_core::InstancePre<RuntimeData>),
    Module(spin_core::ModuleInstancePre<RuntimeData>),
}

pub enum CommandInstance {
    Component(spin_core::Instance),
    Module(spin_core::ModuleInstance),
}

#[async_trait]
impl TriggerExecutor for CommandTrigger {
    const TRIGGER_TYPE: &'static str = "command";
    type RuntimeData = RuntimeData;
    type TriggerConfig = CommandTriggerConfig;
    type RunConfig = NoArgs;
    type InstancePre = CommandInstancePre;

    async fn new(engine: TriggerAppEngine<Self>) -> Result<Self> {
        let components = engine
            .trigger_configs()
            .map(|(_, config)| Component {
                id: config.component.clone(),
                executor: config.executor.clone(),
            })
            .collect();
        Ok(Self { engine, components })
    }

    async fn run(self, _config: Self::RunConfig) -> Result<()> {
        self.handle().await
    }
}

#[async_trait]
impl TriggerInstancePre<RuntimeData, CommandTriggerConfig> for CommandInstancePre {
    type Instance = CommandInstance;

    async fn instantiate_pre(
        engine: &Engine<RuntimeData>,
        component: &AppComponent,
        config: &CommandTriggerConfig,
    ) -> Result<CommandInstancePre> {
        if let CommandExecutorType::Preview1 = &config.executor {
            let module = component.load_module(engine).await?;
            Ok(CommandInstancePre::Module(
                engine.module_instantiate_pre(&module)?,
            ))
        } else {
            let comp = component.load_component(engine).await?;
            Ok(CommandInstancePre::Component(
                engine.instantiate_pre(&comp)?,
            ))
        }
    }

    async fn instantiate(&self, store: &mut Store) -> Result<CommandInstance> {
        match self {
            CommandInstancePre::Component(pre) => pre
                .instantiate_async(store)
                .await
                .map(CommandInstance::Component),
            CommandInstancePre::Module(pre) => pre
                .instantiate_async(store)
                .await
                .map(CommandInstance::Module),
        }
    }
}

impl CommandTrigger {
    pub async fn handle(&self) -> Result<()> {
        let component = &self.components[0];
        match component.executor {
            CommandExecutorType::Preview2 => {
                let (instance, mut store) = self.engine.prepare_instance(&component.id).await?;
                let CommandInstance::Component(instance) = instance else {
                    unreachable!()
                };
                let handler =
                    wasmtime_wasi::preview2::command::Command::new(&mut store, &instance)?;
                let _ = handler.wasi_cli_run().call_run(store).await?;
            }
            CommandExecutorType::Preview1 => {
                let store_builder = self
                    .engine
                    .store_builder(&component.id, spin_core::WasiVersion::Preview1)?;
                let (instance, mut store) = self
                    .engine
                    .prepare_instance_with_store(&component.id, store_builder)
                    .await?;

                let CommandInstance::Module(instance) = instance else {
                    unreachable!()
                };

                let start = instance
                    .get_func(&mut store, "_start")
                    .context("Expected component to export _start function")?;

                let _ = start.call_async(&mut store, &[], &mut []).await?;
            }
        };
        Ok(())
    }
}
