use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Args;
use serde::{Deserialize, Serialize};
use spin_factor_wasi::WasiFactor;
use spin_factors::RuntimeFactors;
use spin_trigger::{Trigger, TriggerApp};

#[derive(Clone)]
pub struct CommandTrigger {
    components: Vec<Component>,
    config: CliArgs,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Component {
    pub id: String,
}

#[derive(Args, Debug, Clone)]
#[clap(trailing_var_arg(true))]
pub struct CliArgs {
    #[clap(multiple_values(true), allow_hyphen_values(true))]
    pub guest_args: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CommandTriggerConfig {
    pub component: String,
}

impl<F: RuntimeFactors> Trigger<F> for CommandTrigger {
    const TYPE: &'static str = "command";

    type CliArgs = CliArgs;

    type InstanceState = ();

    fn new(cli_args: Self::CliArgs, app: &spin_trigger::App) -> anyhow::Result<Self> {
        let components: Vec<Component> = app
            .trigger_configs::<CommandTriggerConfig>(<Self as Trigger<F>>::TYPE)?
            .into_iter()
            .map(|(_, config)| Component {
                id: config.component.clone(),
            })
            .collect();

        if components.len() > 1 {
            tracing::warn!(
                "Multiple components found for command trigger, only the first one will be used"
            );
        }

        if components.is_empty() {
            return Err(anyhow::anyhow!(
                "No components found for command trigger, exiting"
            ));
        }

        Ok(Self {
            components,
            config: cli_args,
        })
    }

    async fn run(self, trigger_app: spin_trigger::TriggerApp<Self, F>) -> anyhow::Result<()> {
        Self::handle(
            self.components
                .first()
                .context("Failed to get the component for the command trigger")?
                .to_owned(),
            trigger_app.into(),
            self.config.clone(),
        )
        .await
    }
}

impl CommandTrigger {
    pub async fn handle<F: RuntimeFactors>(
        component: Component,
        trigger_app: Arc<TriggerApp<Self, F>>,
        args: CliArgs,
    ) -> Result<()> {
        let mut instance_builder = trigger_app.prepare(&component.id)?;
        if let Some(wasi) = instance_builder.factor_builder::<WasiFactor>() {
            let args = std::iter::once(component.id).chain(args.guest_args);
            wasi.args(args);
        }

        let (instance, mut store) = instance_builder.instantiate(()).await?;

        let func = {
            let instance_export = instance
                .get_export(&mut store, None, "wasi:cli/run@0.2.0")
                .context("failed to find the wasi:cli/run@0.2.0 instance in component")?;

            let func_export = instance
                .get_export(&mut store, Some(&instance_export), "run")
                .context("failed to find the \"run\" function in wasi:cli/run@0.2.0 instance")?;

            instance
                .get_typed_func::<(), (Result<(), ()>,)>(&mut store, func_export)
                .context("failed to get typed \"run\" function")?
        };
        let _ = func.call_async(&mut store, ()).await?;

        Ok(())
    }
}
