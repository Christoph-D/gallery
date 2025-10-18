FROM mcr.microsoft.com/devcontainers/rust:2.0-trixie

ENV DEBIAN_FRONTEND=noninteractive

RUN apt-get update && apt-get install -y \
  # Required dependencies
  imagemagick

RUN ln -sf /usr/share/zoneinfo/hostlocaltime /etc/localtime && \
  echo 'en_US.UTF-8 UTF-8' > /etc/locale.gen && \
  dpkg-reconfigure locales && \
  update-locale LANG=en_US.UTF-8

RUN SNIPPET="export PROMPT_COMMAND='history -a' && export HISTFILE=/commandhistory/.bash_history" \
  && mkdir /commandhistory \
  && touch /commandhistory/.bash_history \
  && chown -R vscode /commandhistory \
  && echo "$SNIPPET" >> "/home/vscode/.bashrc"

USER vscode
WORKDIR /home/vscode

ENV CARGO_TARGET_DIR=/home/vscode/target

RUN curl -fsSL https://claude.ai/install.sh | bash && \
  mkdir -p /home/vscode/claude-config
ENV CLAUDE_CONFIG_DIR=/home/vscode/claude-config

WORKDIR /workspaces/gallery
