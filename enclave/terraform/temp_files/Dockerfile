FROM amazonlinux:2

# Install required packages
RUN yum update -y && \
    yum install -y python3-pip python3-devel gcc && \
    yum clean all

# Install Python packages
COPY requirements.txt .
RUN pip3 install -r requirements.txt

# Copy training script
COPY mnist_train.py .

# Set working directory
WORKDIR /app

# Run training script
CMD ["python3", "mnist_train.py"]
