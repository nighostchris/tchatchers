use crate::components::common::FileAttacher;
use crate::components::common::FormButton;
use crate::components::common::WaitingForResponse;
use crate::router::Route;
use crate::utils::jwt::get_jwt_public_part;
use crate::utils::requester::Requester;
use gloo_net::http::Request;
use tchatchers_core::user::PartialUser;
use tchatchers_core::user::UpdatableUser;
use wasm_bindgen::JsCast;
use web_sys::HtmlInputElement;
use yew::{html, Callback, Component, Context, Html, NodeRef, Properties};
use yew_router::history::History;
use yew_router::scope_ext::RouterScopeExt;

pub enum Msg {
    UploadNewPfp(Option<js_sys::ArrayBuffer>),
    PfpUpdated(String),
    SubmitForm,
    ErrorFromServer(String),
    ProfileUpdated,
    FetchJwt,
}

#[derive(Clone, PartialEq, Eq, Properties)]
pub struct Props;

#[derive(Default)]
pub struct Settings {
    user: PartialUser,
    name: NodeRef,
    pfp: Option<String>,
    wait_for_api: bool,
    server_error: Option<String>,
    ok_msg: Option<String>,
}

impl Component for Settings {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        ctx.link().send_message(Msg::FetchJwt);
        Self { ..Self::default() }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::FetchJwt => {
                let window = web_sys::window().unwrap();
                let document = window.document().unwrap();
                let html_document = document.dyn_into::<web_sys::HtmlDocument>().unwrap();
                let document_cookies = html_document.cookie().unwrap();
                let cookies = &mut document_cookies.split(';');
                let mut user: PartialUser = PartialUser::default();
                for cookie in cookies.by_ref() {
                    if let Some(i) = cookie.find('=') {
                        let (key, val) = cookie.split_at(i + 1);
                        if key == "jwt=" {
                            user = get_jwt_public_part(val.into());
                        }
                    }
                }
                if user == PartialUser::default() {
                    ctx.link().history().unwrap().push(Route::SignIn);
                }
                self.user = user.clone();
                self.pfp = user.pfp;
                true
            }
            Msg::SubmitForm => {
                self.wait_for_api = true;
                self.ok_msg = None;
                self.server_error = None;
                if let Some(name) = self.name.cast::<HtmlInputElement>() {
                    if name.check_validity() {
                        let payload = UpdatableUser {
                            id: self.user.id,
                            name: name.value(),
                            pfp: self.pfp.clone(),
                        };
                        let mut req = Requester::<UpdatableUser>::put("/api/user");
                        req.is_json(true).body(Some(payload));
                        let link = ctx.link().clone();
                        self.wait_for_api = true;
                        wasm_bindgen_futures::spawn_local(async move {
                            let resp = req.send().await;
                            if resp.status().is_success() {
                                link.send_message(Msg::ProfileUpdated);
                            } else {
                                link.send_message(Msg::ErrorFromServer(resp.text().await.unwrap()));
                            }
                        });
                    }
                }
                true
            }
            Msg::UploadNewPfp(pfp) => {
                self.wait_for_api = true;
                let req = Request::post("/api/pfp").body(pfp.unwrap()).send();
                let link = ctx.link().clone();
                wasm_bindgen_futures::spawn_local(async move {
                    let resp = req.await.unwrap();
                    if resp.ok() {
                        link.send_message(Msg::PfpUpdated(resp.text().await.unwrap()));
                    } else {
                        link.send_message(Msg::ErrorFromServer(resp.text().await.unwrap()));
                    }
                });
                true
            }
            Msg::ErrorFromServer(s) => {
                self.wait_for_api = false;
                self.ok_msg = None;
                self.server_error = Some(s);
                true
            }
            Msg::PfpUpdated(pfp_path) => {
                self.wait_for_api = false;
                self.pfp = Some(pfp_path);
                true
            }
            Msg::ProfileUpdated => {
                self.wait_for_api = false;
                self.ok_msg = Some("Your profile has been updated with success.".into());
                ctx.link().send_message(Msg::FetchJwt);
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let pfp = match &self.user.pfp {
            None => match &self.pfp {
                Some(_) => {
                    html! {<span>{"Your new profile picture is ready to be uploaded"}</span>}
                }
                None => html! { <span>{"You don't have any profile picture so far"}</span> },
            },
            Some(v) => html! { <><img class="h-10 w-10 rounded-full" src={v.clone()} /></> },
        };
        let link = ctx.link().clone();
        let end_of_form = match self.wait_for_api {
            true => html! { <WaitingForResponse /> },
            false => html! { <FormButton label="Update" /> },
        };
        html! {
            <>
                <div class="flex items-center justify-center h-full">
                <form class="w-full max-w-sm border-2 px-6 py-6  lg:py-14" onsubmit={ctx.link().callback(|_| Msg::SubmitForm)} action="javascript:void(0);" >

                <h2 class="text-xl mb-10 text-center text-gray-500 font-bold">{"Settings"}</h2>
                  <div class="md:flex md:items-center mb-6">
                    <div class="md:w-1/3">
                      <label class="block text-gray-500 font-bold md:text-right mb-1 md:mb-0 pr-4" for="inline-full-name">
                      {"Your login"}
                      </label>
                    </div>
                    <div class="md:w-2/3">
                      <input class="peer bg-gray-200 appearance-none border-2 border-gray-200 rounded w-full py-2 px-4 text-gray-700 leading-tight focus:outline-none focus:bg-white focus:border-purple-500 focus:invalid:border-red-500 visited:invalid:border-red-500" id="inline-full-name" type="text" required=true minlength="3" value={self.user.login.clone()} disabled=true/>
                    </div>
                    </div>
                  <div class="md:flex md:items-center mb-6">
                    <div class="md:w-1/3">
                      <label class="block text-gray-500 font-bold md:text-right mb-1 md:mb-0 pr-4" for="inline-full-name">
                      {"Your name"}
                      </label>
                    </div>
                    <div class="md:w-2/3">
                      <input class="peer bg-gray-200 appearance-none border-2 border-gray-200 rounded w-full py-2 px-4 text-gray-700 leading-tight focus:outline-none focus:bg-white focus:border-purple-500 focus:invalid:border-red-500 visited:invalid:border-red-500" id="inline-full-name" type="text" required=true minlength="3" ref={&self.name} value={self.user.name.clone()}/>
                    </div>
                  </div>
                  <div class="md:flex md:items-center mb-6">
                    <div class="md:w-1/3">
                      <label class="block text-gray-500 font-bold md:text-right mb-1 md:mb-0 pr-4" for="inline-full-name">
                      {"Your profile picture"}
                      </label>
                    </div>
                    <div class="md:w-2/3 flex justify-center items-center">
                    {pfp}
                    <FileAttacher disabled=false accept={Some(String::from(".png,.webp,.jpg,.jpg"))} on_file_attached={Callback::from(move |file_path: Option<js_sys::ArrayBuffer>| {
                        link.send_message(Msg::UploadNewPfp (file_path));
        })}/>
                    </div>
                  </div>
                  <small class="flex mt-4 mb-2 items-center text-red-500" hidden={self.server_error.is_none()}>
                    {self.server_error.as_ref().unwrap_or(&String::new())}
                  </small>
                  <small class="flex mt-4 mb-2 items-center text-green-500" hidden={self.ok_msg.is_none()}>
                    {self.ok_msg.as_ref().unwrap_or(&String::new())}
                  </small>
                  {end_of_form}
                </form>
                </div>
            </>
        }
    }
}
