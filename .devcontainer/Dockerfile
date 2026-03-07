FROM mcr.microsoft.com/devcontainers/rust:2.0-trixie

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y imagemagick

RUN ln -sf /usr/share/zoneinfo/hostlocaltime /etc/localtime && \
  echo 'en_US.UTF-8 UTF-8' > /etc/locale.gen && \
  dpkg-reconfigure locales && \
  update-locale LANG=en_US.UTF-8

USER vscode
WORKDIR /home/vscode

RUN mkdir -p /home/vscode/persistent-config

RUN SNIPPET="export PROMPT_COMMAND='history -a' && export HISTFILE=/home/vscode/persistent-config/.bash_history" \
  && echo "$SNIPPET" >> "/home/vscode/.bashrc"

ENV CARGO_TARGET_DIR=/home/vscode/target

ENV EDITOR='code --wait'
WORKDIR /workspaces/gallery
