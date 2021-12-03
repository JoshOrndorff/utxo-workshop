# This is an educational attempt to create a docker image that runs the utxo-workshop
# I found these resources helpful
# https://docs.docker.com/get-started/part2/
# https://blog.sedrik.se/posts/my-docker-setup-for-rust/

# This could be helpful for building and publishing the image from github actions
# https://github.com/docker/build-push-action

# Choose the base image
# I think there is also a parity rust builder image
#FROM alpine:latest # Alpine gives a nice small image (46MB compared to ubuntu's 114MB) but the container doesn't actually run...
#FROM ubuntu:20.04
FROM debian:latest # Works fine, but 155MB (even bigger than ubuntu)

# Set the working directory.
# WORKDIR /usr/src/app

# Copy the node into the image
COPY target/release/utxo-workshop .

# Strip the binary. I hope this makes the resulting image smaller.
# I'm not sure of the consequences
# Actually, stripping only took it from 39MB -> 30MB and the `strip` command isn't
# included in alpine or ubuntu base images. I say skip the stripping.
#RUN strip utxo-workshop


# Open some ports
EXPOSE 30333 9933 9944

# Tutorial recommended specifying the command like this.
# But I couldn't pass arguments or subcommands to the node that way
# CMD [ "./utxo-workshop" ]

# Specifying an ENTRYPOINT rather than a CMD allows me to pass args to the node
# https://stackoverflow.com/a/29661891/4184410
ENTRYPOINT ["./utxo-workshop"]

# TODO Maybe we could copy a UI into the image too
