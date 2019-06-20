# Blog

This is an overbuilt, yet simplistic, blog setup to explore the use of some Rust async libraries, notably [tide](https://github.com/rustasync/tide).

A CLI is provided to manage the contents on the blog, in lieu of a web-based editor.

This project is structured as a cargo workspace, with the server component living as `nanoblog`, and the CLI as `blogctl`.


# `nanoblog` (server)
The server component is provided with a Dockerfile and associated Helm charts to deploy to a kubernetes cluster.
Redis is the datastore used for both the bearer tokens and the posts.

## Local Setup
1. Install `minikube`, `docker`, and `helm`
1. Point docker client at minikube server: `eval $(minikube docker-env)`
1. Build image: `docker build nanoblog -t <your_tag_here>`
1. Generate a redis password: `kc create secret generic nanoblog-redis --from-literal=redis-password=$(< /dev/urandom tr -dc _A-Z-a-z-0-9 | head -c${1:-32};echo)`
1. Install/upgrade with helm chart: `helm upgrade -i nanoblog ./nanoblog/charts/nanoblog --set image.tag=<your_tag_here>`


# `blogctl` (CLI)
The CLI is a simple rust project to interface with the blog server and manage posts and their contents.

## Local Setup
1. `cargo build --release`
1. Set the `hostname` and `token` properties in the following location: `~/.config/blogctl/config.json`.
1. `/target/release/blogctl -h` should reveal the rest of the documentation needed!
