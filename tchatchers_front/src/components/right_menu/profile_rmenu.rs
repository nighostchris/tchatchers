use std::rc::Rc;

use serde::{Deserialize, Serialize};
use tchatchers_core::profile::Profile;
use yew::{function_component, html, use_context, Html, Properties};
use yew_agent::Dispatched;

use crate::{
    components::{common::I18N, toast::Alert},
    services::toast_bus::ToastBus,
    utils::{client_context::ClientContext, requester::Requester},
};

#[derive(Properties, PartialEq, Serialize, Deserialize, Debug, Clone)]
pub struct ProfileRMenuProps {
    pub user_id: i32,
}

#[function_component(ProfileRMenu)]
pub fn profile_rmenu(props: &ProfileRMenuProps) -> Html {
    let client_context = use_context::<Rc<ClientContext>>().unwrap();

    let bearer = client_context.bearer.clone();

    let translation = client_context.translation.clone();

    let revoke_user_li = {
        let bearer = bearer.clone();
        let translation = translation.clone();
        let revoke_user_id = {
            let props = props.clone();
            move |_| {
                let mut req = Requester::delete(&format!("/api/user/revoke/{}", props.user_id));
                req.bearer(bearer.clone());
                wasm_bindgen_futures::spawn_local(async move {
                    req.send().await;
                })
            }
        };
        html! {
            <li class="hover:text-gray-100" onclick={revoke_user_id}>
                <I18N label={"revoke_user"} default={"Revoke_user"} {translation}/>
            </li>
        }
    };

    let report_user_li = {
        let report_user_id = {
            let props = props.clone();
            move |_| {
                let mut req = Requester::post(&format!("/api/user/{}/report", props.user_id));
                req.bearer(bearer.clone());
                wasm_bindgen_futures::spawn_local(async move {
                    let res = req.send().await;
                    if res.ok() {
                        ToastBus::dispatcher().send(Alert {
                            is_success: true,
                            content: "This user has been reported with success".into(),
                        });
                    } else {
                        ToastBus::dispatcher().send(Alert {
                            is_success: false,
                            content: res.text().await.unwrap_or_else(|_| {
                                "A problem happened while reporting this user".into()
                            }),
                        });
                    }
                })
            }
        };
        html! {
        <li class="hover:text-gray-300" onclick={report_user_id}>
            <I18N label={"report_user"} default={"Report user"} {translation}/>
        </li>}
    };

    match client_context.user.as_ref().unwrap().profile {
        Profile::Moderator | Profile::Admin => html! {
            <ul>
                {revoke_user_li}
            </ul>
        },
        _ => html! {
            <ul>
                {report_user_li}
            </ul>
        },
    }
}
