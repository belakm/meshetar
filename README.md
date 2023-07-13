# 🫰 MESHETAR

An algotrading system to be. Currently supports fetching history + predicting signals with R. 

Features a server running on 🚀 Rocket and an app build with 🍁 Sycamore.

## How to run

#### Server

0. `cd` into `server`
1. Run `Rscript renv_prepare.R` to install R dependencies.
2. Run  `cargo watch -x run -p server` to start the Rocket server and other services. Alternatively run `cargo build` and `cargo run` if you dont need hot reload.

#### App

0. `cd` into `server`
1. Run `trunk serve`

## Screenshot

![image](https://github.com/belakm/meshetar/assets/13392444/0ec4b2bf-8cdb-4d54-b9fb-e5edb59b4106)
