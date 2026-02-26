FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    openssl \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

RUN groupadd -r appgroup && useradd -r -g appgroup appuser
USER appuser

ARG TARGETARCH

COPY artifacts/necko3-${TARGETARCH} /usr/local/bin/necko3

USER root
RUN chmod +x /usr/local/bin/necko3
USER appuser

CMD ["necko3"]