FROM golang:1.24.3-alpine

WORKDIR /app/fizzbuzz

EXPOSE 8080

COPY . .

RUN go build -o build/fizzbuzz

CMD ["./build/fizzbuzz", "serve"]
