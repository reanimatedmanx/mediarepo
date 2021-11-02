use mediarepo_api::types::misc::InfoResponse;
use mediarepo_core::rmp_ipc::prelude::*;

mod from_model;
mod namespaces;
mod utils;

pub fn get_builder(address: &str) -> IPCBuilder {
    namespaces::build_namespaces(IPCBuilder::new().address(address)).on("info", callback!(info))
}

#[tracing::instrument(skip_all)]
async fn info(ctx: &Context, event: Event) -> IPCResult<()> {
    let response = InfoResponse::new(
        env!("CARGO_PKG_NAME").to_string(),
        env!("CARGO_PKG_VERSION").to_string(),
    );
    ctx.emitter
        .emit_response(event.id(), "info", response)
        .await?;

    Ok(())
}
