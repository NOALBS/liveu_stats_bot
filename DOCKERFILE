# Use an official Ubuntu as a parent image
FROM ubuntu:latest

# Set the maintainer label
LABEL maintainer="rsmith@netandvet.com"

# Set environment variables to non-interactive (this prevents some prompts)
ENV DEBIAN_FRONTEND=noninteractive

# Run package updates and install packages
RUN apt-get update \
    && apt-get install -y \
    git \
    python3 \
    python3-pip \
    wget

# Clone the repository
RUN wget https://github.com/715209/liveu_stats_bot/releases/download/v0.6.0/liveu_stats_bot-x86_64-unknown-linux-gnu.tar.gz

RUN tar -xzf liveu_stats_bot-x86_64-unknown-linux-gnu.tar.gz
# Change the working directory
WORKDIR liveu_stats_bot

# Install any needed packages specified in requirements.txt
RUN pip3 install --trusted-host pypi.python.org -r requirements.txt

# Make port 80 available to the world outside this container (if needed)
#EXPOSE 80

# Define environment variable (if needed)
# ENV NAME=World

# Run the application
# Note: Update this according to the correct command to run your application
CMD ["/bin/bash", "liveu_stats_bot"]
