FROM devenv-base:latest

# install additional dependencies
RUN sudo apk add  \
  rustup          \
  --no-cache

# set up rust for dev user
RUN /usr/bin/rustup-init -y
RUN echo '. "$HOME/.cargo/env"' >> $HOME/.profile
