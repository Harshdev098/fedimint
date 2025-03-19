use axum::extract::{Form, State};
use axum::response::{IntoResponse, Redirect};
use axum_extra::extract::cookie::CookieJar;
use fedimint_core::util::SafeUrl;
use fedimint_server_core::dashboard_ui::{DashboardApiModuleExt, DynDashboardApi};
use maud::{Markup, html};

use crate::{AuthState, check_auth};

// Form for gateway management
#[derive(serde::Deserialize)]
pub struct GatewayForm {
    pub gateway_url: SafeUrl,
}

// Function to render the Lightning V2 module UI section
pub async fn render(lightning: &fedimint_lnv2_server::Lightning) -> Markup {
    let gateways = lightning.gateways_ui().await;
    let consensus_block_count = lightning.consensus_block_count_ui().await;
    let consensus_unix_time = lightning.consensus_unix_time_ui().await;
    let formatted_unix_time = chrono::DateTime::from_timestamp(consensus_unix_time as i64, 0)
        .map(|dt| dt.to_rfc2822())
        .unwrap_or("Invalid time".to_string());

    html! {
        div class="row gy-4 mt-2" {
            div class="col-12" {
                div class="card h-100" {
                    div class="card-header dashboard-header" { "Lightning V2" }
                    div class="card-body" {
                        // Consensus status information
                        div class="mb-4" {
                            table class="table" {
                                tr {
                                    th { "Consensus Block Count" }
                                    td { (consensus_block_count) }
                                }
                                tr {
                                    th { "Consensus Unix Time" }
                                    td { (formatted_unix_time) }
                                }
                            }
                        }

                        // Gateway management
                        div {
                            h5 { "Gateway Management" }
                            // Add new gateway form
                            div class="mb-4" {
                                form action="/lnv2_gateway_add" method="post" class="row g-3" {
                                    div class="col-md-9" {
                                        div class="form-group" {
                                            div class="text-muted mb-1" style="font-size: 0.875em;" {
                                                "Please enter a valid URL starting with http:// or https://"
                                            }
                                            input
                                                type="url"
                                                class="form-control"
                                                id="gateway-url"
                                                name="gateway_url"
                                                placeholder="Enter gateway URL"
                                                pattern="https?:\\/\\/(www\\.)?[-a-zA-Z0-9@:%._\\+~#=]{2,256}\\.[a-z]{2,6}\\b([-a-zA-Z0-9@:%_\\+.~#?&//=]*)"
                                                required;
                                        }
                                    }
                                    div class="col-md-3" {
                                        div style="margin-top: 24px;" {
                                            button type="submit" class="btn btn-primary w-100 form-control" { "Add Gateway" }
                                        }
                                    }
                                }
                            }

                            // Gateway list
                            @if gateways.is_empty() {
                                div class="alert alert-info" { "No gateways configured yet." }
                            } @else {
                                div class="table-responsive" {
                                    table class="table table-hover" {
                                        thead {
                                            tr {
                                                th { "Gateway URL" }
                                                th class="text-end" { "Actions" }
                                            }
                                        }
                                        tbody {
                                            @for gateway in &gateways {
                                                tr {
                                                    td {
                                                        a href=(gateway.to_string()) target="_blank" { (gateway.to_string()) }
                                                    }
                                                    td class="text-end" {
                                                        form action="/lnv2_gateway_remove" method="post" style="display: inline;" {
                                                            input type="hidden" name="gateway_url" value=(gateway.to_string());
                                                            button type="submit" class="btn btn-sm btn-danger" {
                                                                "Remove"
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

// Handler for adding a new gateway
pub async fn add_gateway(
    State(state): State<AuthState<DynDashboardApi>>,
    jar: CookieJar,
    Form(form): Form<GatewayForm>,
) -> impl IntoResponse {
    if !check_auth(&state.auth_cookie_name, &state.auth_cookie_value, &jar).await {
        return Redirect::to("/login").into_response();
    }

    state
        .api
        .get_module::<fedimint_lnv2_server::Lightning>()
        .expect("Route only mounted when Lightning V2 module exists")
        .add_gateway_ui(form.gateway_url)
        .await;

    Redirect::to("/").into_response()
}

// Handler for removing a gateway
pub async fn remove_gateway(
    State(state): State<AuthState<DynDashboardApi>>,
    jar: CookieJar,
    Form(form): Form<GatewayForm>,
) -> impl IntoResponse {
    if !check_auth(&state.auth_cookie_name, &state.auth_cookie_value, &jar).await {
        return Redirect::to("/login").into_response();
    }

    state
        .api
        .get_module::<fedimint_lnv2_server::Lightning>()
        .expect("Route only mounted when Lightning V2 module exists")
        .remove_gateway_ui(form.gateway_url)
        .await;

    Redirect::to("/").into_response()
}
