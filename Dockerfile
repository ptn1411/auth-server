# Auth Server Docker Image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -r -s /bin/false authserver

# Create directories
RUN mkdir -p /app/keys && chown -R authserver:authserver /app

WORKDIR /app

# Copy binary from build context (downloaded artifact)
COPY docker/auth-server-linux-amd64 /app/auth-server
RUN chmod +x /app/auth-server

# Switch to non-root user
USER authserver

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Run
CMD ["/app/auth-server"]
