![ci.yml](https://img.shields.io/github/actions/workflow/status/nag763/tchatchers/ci.yml)
[![tchatchers-stars](https://img.shields.io/github/stars/nag763/tchatchers?style=social)](https://github.com/nag763/tchatchers/stargazers)
[![tchatchers-license](https://img.shields.io/github/license/nag763/tchatchers)](https://raw.githubusercontent.com/nag763/tchatchers/main/LICENSE.MD)
[![github-issues](https://img.shields.io/github/issues/nag763/tchatchers)](https://github.com/nag763/tchatchers/issues)
[![instance-health](https://img.shields.io/website?down_color=red&down_message=down&label=public%20instance&up_color=green&up_message=up&url=https%3A%2F%2Ftchatche.xyz)](https://tchatche.xyz)

<p align="center"><img height="300" src="https://raw.githubusercontent.com/nag763/tchatchers/main/.github/gh_logo.png"></img></p>

<h2 align="center">tchatchers</h2>
<h4 align="center">A blazing fast chat application built with Axum and yew.rs :rocket:</h4>

<p align="center"><img src="https://raw.githubusercontent.com/nag763/tchatchers/main/.github/app_screens.png"></img></p>

## TL;DR

* :speech_balloon: tchatchers is a realtime chat application built with yew.rs (frontend) and axum (backend) frameworks.
* :heavy_check_mark: Easy to use, only requires authentication informations to be used.
* :rocket: Blazing fast, completely built on Rust.
* :moon: Supports browser's dark mode.
* :sparkles: Simple yet elegant UI.
* :book: Translated in several languages.
* :lock: With little data collected and robust security, this app is completly secured.
* :ship: This application is fully optimized to be as small as possible for production environments.

## How to access the application

The application is deployed on https://tchatche.xyz and should be compatible with any modern navigator.

It was formerly on tchatchers but the domain name renewal fee was too expensive.

## About

tchatchers is an application used to help clients to communicate between each others. It is built with yew.rs and axum server in order to provide blazing fast responses and strong API.

The main application's usage is to create rooms to talk between people being connected to the application. You like football ? Try the football room. You like philosophy ? Try the philosophy one.

All depends on you to chat how you want to .

## :new: Installing the application locally and starting developping

Follow [this guide](./SETUP.md). 

## Project structure

```
.
├── Cargo.lock => Dependency lock file generated by Cargo
├── Cargo.toml => Main configuration file for the Rust project
├── CODE_OF_CONDUCT.md => Code of conduct for contributors
├── docker-compose_dev.yml => Docker compose file for development environment
├── docker-compose.yml => Docker compose file for production environment
├── Dockerfile_back => Dockerfile for building the backend service
├── Dockerfile_front => Dockerfile for building the frontend service
├── Dockerfile_tct => Dockerfile for building the TCT tool
├── LICENSE.MD => License file for the project
├── Makefile.toml => Makefile for building and testing the project
├── README.md => Project README file
├── SETUP.md => Instructions for setting up the development environment
├── setup.sh => Shell script for setting up the development environment
├── tchatchers_back => Rust crate for the backend service
├── tchatchers_cli_tools => Rust crate for command-line tools
├── tchatchers_core => Rust crate for shared core functionality
└── tchatchers_front => Rust crate for the frontend service
└── tchatchers_async => Rust crate for the asynchronous service
```

## Rustdoc

The rustdoc can be found for each subproject at :

- tchatchers_core : [here](https://tchatche.xyz/doc/tchatchers_core/) 
- tchatchers_back : [here](https://tchatche.xyz/doc/tchatchers_back/)
- tchatchers_front : [here](https://tchatche.xyz/doc/tchatchers_front/)
- tchatchers_async : [here](https://tchatche.xyz/doc/tchatchers_async/)
- tct (tchatchers_cli_tool): [here](https://tchatche.xyz/doc/tct/)

## Technologies used

|Technology/Framework|Utility                     |Version|
|--------------------|----------------------------|-------|
|Rust                |Programming language        |1.68.2 |
|Tailwind            |Stylesheets                 |3.X    |
|yew.rs              |WASM Frontend framework     |0.20   |
|axum                |Rust server                 |0.6.12 |
|trunk-rs            |Rust development WASM server|0.16   |
|nginx               |Reverse proxy server        |latest |
|Postgres            |SQL engine                  |latest |
|Redis               |Messaging and cache         |latest |

## CICD

![](https://raw.githubusercontent.com/nag763/tchatchers/main/.github/cicd.svg)

## Production project architecture

![](https://raw.githubusercontent.com/nag763/tchatchers/main/.github/application_schema.png)

The production architecture consists of several layers :
* <u>The client</u>, who uses the application and interacts with the different ressources.
* <u>The proxy layer</u>, that defines some security constraints such as BruteForce mitigation, the HTTPS connection, the read time out and HTTP headers. This layer is the sole entry point for the client to the application, as others aren't publicly reachable since they are on another network.
* <u>The applicative layer :</u> This contains two noticeable applicative layers :
    - First, the front layer, a static WASM file being downloaded once by the client and then used to display the application's data to the client. Understand that there is no server side rendering.

- <u>The Asynchronous Layer</u>: This layer, introduced in the updated architecture, is responsible for processing queued messages stored in Redis. It includes components such as the asynchronous payload, processor, and queue modules. These modules facilitate the retrieval, processing, and deletion of messages from the Redis queue.
    - Secondly,the API layer, used to persist the application data, besides of permitting operations such as authentication and translation.
* <u>The data layer :</u> Mainly used for persistence. The choice has been made to persist all the data onto a Postgres database, both the user's data and the chats. On network level, the data layer can only be accessed by the API layer, and is not exposed publicly. Redis on its side mainly stores the authorization and refresh tokens.


## Postgres schema

Made with one of my other tools, [doteur](https://github.com/nag763/doteur).

![img](https://raw.githubusercontent.com/nag763/tchatchers/main/.github/schema.jpeg)

## Personnal objectives behind this project, feedback

My goal with this project was to learn more about both WASM and Websocket technologies besides of perfecting my Rust knowledge. It was really nice to build such a project and I hope it can inspire or help other peoples to built similar tools. I think the code is pretty good (even though it should be perfectible) on the back and core projects, but quite perfectible on the front one. It is to note that there are still some bugs remaining.

My feeling is that both Rust and WASM have a pretty good future when it comes to frontend apps. The backend side of Rust already has several frameworks that are making it a reliable language when it comes to handling data logic, and the development of yew.rs or similar technologies in the future could be a game changer for the interest users and firms have toward the language. 

## Special thanks

Thanks to the Rust community for all their advices and their time reviewing my project, it's always a pleasure to be part of such a community. :blush: :crab:
