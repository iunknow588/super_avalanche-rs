use crate::proto::pb::google::protobuf::Empty;
use crate::proto::pb::vm::{
    CrossChainAppRequestFailedMsg, CrossChainAppRequestMsg, CrossChainAppResponseMsg,
};
use tonic::{Request, Response, Status};

impl<V> super::server::Server<V> {
    pub async fn cross_chain_app_request(
        &self,
        _request: Request<CrossChainAppRequestMsg>,
    ) -> Result<Response<Empty>, Status> {
        // TODO: 实现跨链请求逻辑
        Ok(Response::new(Empty {}))
    }

    pub async fn cross_chain_app_request_failed(
        &self,
        _request: Request<CrossChainAppRequestFailedMsg>,
    ) -> Result<Response<Empty>, Status> {
        // TODO: 实现跨链请求失败逻辑
        Ok(Response::new(Empty {}))
    }

    pub async fn cross_chain_app_response(
        &self,
        _request: Request<CrossChainAppResponseMsg>,
    ) -> Result<Response<Empty>, Status> {
        // TODO: 实现跨链响应逻辑
        Ok(Response::new(Empty {}))
    }
}
