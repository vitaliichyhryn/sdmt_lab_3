FROM golang:1.24.3-alpine AS builder

WORKDIR /build

COPY . .

RUN go mod download
RUN go build -o fizzbuzz

FROM gcr.io/distroless/static:nonroot

WORKDIR /app

COPY --from=builder /build/fizzbuzz .
COPY --from=builder /build/templates ./templates

EXPOSE 8080

CMD ["/app/fizzbuzz", "serve"]
