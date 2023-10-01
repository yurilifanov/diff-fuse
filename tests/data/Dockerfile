FROM docker.io/ubuntu:22.04

ENV DEBIAN_FRONTEND=noninteractive
ARG PASSWD=password

RUN apt-get update && \
    apt-get install -y subversion vim && \
    apt-get clean && \
    rm -rf /var/cache/apt/archives /var/lib/apt/lists/*

RUN svnadmin create /svn && \
    echo '[general]' > /svn/conf/svnserve.conf && \
    echo 'anon-access = write' >> /svn/conf/svnserve.conf && \
    mkdir -p /src && \
    svn import -m 'Init' /src file:///svn/src && \
    rm -rf /src

COPY entrypoint.sh entrypoint.sh
ENTRYPOINT [ "bash", "/entrypoint.sh" ]
WORKDIR /src
