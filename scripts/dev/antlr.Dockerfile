FROM eclipse-temurin:17-jre-jammy

ARG ANTLR_VERSION=4.13.2
ARG ANTLR_SHA256=eae2dfa119a64327444672aff63e9ec35a20180dc5b8090b7a6ab85125df4d76

RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates curl \
    && rm -rf /var/lib/apt/lists/*

RUN mkdir -p /opt/antlr \
    && curl --fail --show-error --silent --location \
        --retry 5 --retry-delay 2 --retry-all-errors \
        --connect-timeout 20 \
        "https://www.antlr.org/download/antlr-${ANTLR_VERSION}-complete.jar" \
        -o /opt/antlr/antlr.jar \
    && echo "${ANTLR_SHA256}  /opt/antlr/antlr.jar" | sha256sum -c -

WORKDIR /work

ENTRYPOINT ["java", "-jar", "/opt/antlr/antlr.jar"]
