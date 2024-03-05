###################
# STAGE 1: builder
###################

# Build currently doesn't work on > Java 11 (i18n utils are busted) so build on 8 until we fix this
FROM debian:stable-slim as builder

WORKDIR /app

ENV FC_LANG en-US
ENV LC_CTYPE en_US.UTF-8
ENV PATH="/root/.cargo/bin:/opt/miniconda/bin:/opt/miniconda/envs/biomedgps/bin:${PATH}"

# bash:    various shell scripts
# wget:    installing lein
# git:     ./bin/version
# make:    backend building
# gettext: translations
RUN apt-get update && apt-get install -y coreutils bash git wget make gettext curl gcc g++ libssl-dev pkg-config

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --default-toolchain stable

RUN wget -O miniconda.sh https://repo.anaconda.com/miniconda/Miniconda3-latest-Linux-x86_64.sh && \
    bash miniconda.sh -b -p /opt/miniconda && \
    rm miniconda.sh
RUN conda create -n biomedgps nodejs=16.13.1
RUN npm install -g yarn@1.22.19

# Add the rest of the source
ADD . .

# Fetch all submodule
RUN cd studio && git submodule update --init --recursive

RUN cd studio && yarn && yarn build:embed

RUN make build-biomedgps

# ###################
# # STAGE 2: runner
# ###################

FROM debian:stable-slim as runner

ENV PATH="$PATH"
ENV PYTHONDONTWRITEBYTECODE=1
ENV FC_LANG en-US
ENV LC_CTYPE en_US.UTF-8

LABEL org.opencontainers.image.source = "https://github.com/yjcyxky/biomedgps"

RUN apt-get update && apt-get install -y coreutils bash git wget

WORKDIR /data

# Customized
COPY --from=builder /app/target/release/biomedgps /usr/local/bin/biomedgps
COPY --from=builder /app/target/release/biomedgps-cli /usr/local/bin/biomedgps-cli

RUN chmod +x /usr/local/bin/biomedgps && chmod +x /usr/local/bin/biomedgps-cli

# Run it
ENTRYPOINT ["biomedgps", "--help"]