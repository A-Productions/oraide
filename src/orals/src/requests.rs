//! Contains LSP request implementations

use lsp_types::request::{
    Initialize as InitRequest,
    HoverRequest,
};

use jsonrpc_core::{
    ErrorCode,
};

use crate::{
    lsp::{
        RequestId,
    },
    dispatch::{
        RequestAction,
    },
    context::{
        Context,
        InitContext,
    },
    server::{
        Response as _,
        self,
        Output,
        NoResponse,
        ResponseError,
        BlockingRequestAction,
    },
};

impl BlockingRequestAction for InitRequest {
    type Response = NoResponse;

    fn handle<O: Output>(
        id: RequestId,
        _params: Self::Params,
        ctx: &mut Context,
        output: O,
    ) -> Result<Self::Response, ResponseError> {
        if ctx.inited().is_ok() {
            return Err(ResponseError::Message(
                // The LSP spec doesn't dictate a code for this scenario
                // so just use a number out-of-thin-air
                ErrorCode::ServerError(123),
                "Already received an `initialize` request".to_owned(),
            ));
        }

        let result = lsp_types::InitializeResult { capabilities: server::capabilities(ctx) };

        // Send the response early, before `ctx.init`, to enforce the
        // initialize-response-before-all-other-messages constraint
        result.send(id, &output);

        // TODO: Change `init` to take a `ClientCapabilities` (maybe?)
        // https://github.com/rust-lang/rls/blob/17a439440e6b00b1f014a49c6cf47752ecae5bb7/rls/src/server/mod.rs#L160
        // let capabilities = lsp_types::ClientCapabilities::new(&params);
        ctx.init(&output).unwrap();

        Ok(NoResponse)
    }
}

impl RequestAction for HoverRequest {
    type Response = lsp_types::Hover;

    fn fallback_response() -> Result<Self::Response, ResponseError> {
        Ok(lsp_types::Hover {
            contents: lsp_types::HoverContents::Array(vec![]),
            range: None,
        })
    }

    fn handle(_ctx: InitContext, params: Self::Params) -> Result<Self::Response, ResponseError> {
        log::trace!("Got hover request in `{}` at `{}:{}`", params.text_document.uri, params.position.line, params.position.character);
        Ok(Self::Response {
            contents: lsp_types::HoverContents::Scalar(
                lsp_types::MarkedString::String("look at you hovering over symbols, go u".to_string())
            ),
            range: None,
        })
    }
}