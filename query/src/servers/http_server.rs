// Copyright 2020 Datafuse Labs.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::net::SocketAddr;

use axum_server::Handle;
use common_base::tokio::task::JoinHandle;
use common_exception::ErrorCode;
use common_exception::Result;

// TODO(youngsofun): refactor http_services in api and metrics to remove duplicated code
pub struct HttpServer {
    join_handle: Option<JoinHandle<std::io::Result<()>>>,
    pub(crate) abort_handler: Handle,
}

impl HttpServer {
    pub(crate) fn create() -> HttpServer {
        HttpServer {
            join_handle: None,
            abort_handler: axum_server::Handle::new(),
        }
    }
    pub async fn start_server(
        &mut self,
        join_handler: JoinHandle<std::io::Result<()>>,
    ) -> Result<SocketAddr> {
        self.join_handle = Some(join_handler);
        self.abort_handler.listening().await;

        match self.abort_handler.listening_addrs() {
            None => Err(ErrorCode::CannotListenerPort("")),
            Some(addresses) if addresses.is_empty() => Err(ErrorCode::CannotListenerPort("")),
            Some(addresses) => {
                // 0.0.0.0, for multiple network interface, we may listen to multiple address
                let first_address = addresses[0];
                for address in addresses {
                    if address.port() != first_address.port() {
                        return Err(ErrorCode::CannotListenerPort(""));
                    }
                }
                Ok(first_address)
            }
        }
    }
    pub async fn shutdown(&mut self) {
        self.abort_handler.graceful_shutdown();

        if let Some(join_handle) = self.join_handle.take() {
            if let Err(error) = join_handle.await {
                log::error!(
                    "Unexpected error during shutdown Http API handler. cause {}",
                    error
                );
            }
        }
    }
}
