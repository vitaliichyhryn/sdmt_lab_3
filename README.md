# Контейнеризація
## Python
### 1. Створення базового Dockerfile з фіксацією залежностей
З метою фіксації версій залежностей, створимо знімок поточного середовища та
запишемо його в окремий файл, на який будемо посилатися у Dockerfile на етапі
встановлення залежностей:
```bash
pip freeze > requirements.txt
```

Базовий знімок, який було використано: `python:3.13`

Dockerfile:
```dockerfile
FROM python:3.13

WORKDIR /app/spaceship

COPY . .

RUN pip install --no-cache-dir -r requirements.txt

EXPOSE 8080

CMD ["uvicorn", "spaceship.main:app", "--host=0.0.0.0", "--port=8080"]
```

Метрики:

| Час збірки | Розмір знімку |
| - | - |
| 25.4s | 1.12GB |

### 2. Перезбірка знімку після внесення змін у файл
Зміни було внесено у файл `build/index.html`.

Метрики:

| Час збірки | Розмір знімку |
| - | - |
| 12.4s | 1.12GB |

Час збірки зайняв вдвічи менше часу, оскільки більше не треба було
завантажувати базовий образ.

З визначених у Dockerfile команд, лише одна була закешована:
```
=> CACHED [2/4] WORKDIR /app/spaceship  0.0s
```

### 3. Оптимізація кешування
Встановлення залежностей було перенесено на початок Dockerfile з метою
оптимізації кешування.

Dockerfile:
```dockerfile
FROM python:3.13

WORKDIR /app/spaceship

COPY requirements.txt .

RUN pip install --no-cache-dir -r requirements.txt

EXPOSE 8080

COPY . .

CMD ["uvicorn", "spaceship.main:app", "--host=0.0.0.0", "--port=8080"]
```

Метрики першої збірки після оптимізації Dockerfile:

| Час збірки | Розмір знімку |
| - | - |
| 12.3s | 1.12GB |

Після першої збірки зміни було знову внесено у файл `build/index.html`.

Метрики:

| Час збірки | Розмір знімку |
| - | - |
| 2.7s | 1.12GB |

З визначених у Dockerfile команд, були закешовані:
```
=> CACHED [2/5] WORKDIR /app/spaceship                              0.0s
=> CACHED [3/5] COPY requirements.txt .                             0.0s
=> CACHED [4/5] RUN pip install --no-cache-dir -r requirements.txt  0.0s
```

### 4. Використання компактнішого базового знімку
Базовий знімок, який було використано: `python:3.13-alpine`

Перед збіркою знімок було завантажено:
```bash
docker pull python:3.13-alpine
```

Метрики:

| Час збірки | Розмір знімку |
| - | - |
| 13.7s | 139MB |

Попри завантаження базового знімку, час збірки зайняв більше часу, оскільки
через використання нового базового знімку кеш було втрачено.

### 5. Підключення сторонніх бібліотек
У файл `spaceship/routers/api.py` було додано новий endpoint, який використовує
нову сторонню бібліотеку `numpy`.

Для виключення впливу кешування, перед кожною збіркою його було видалено:
```bash
docker builder prune --all
```

Метрики для `python:3.13`:

| Час збірки | Розмір знімку |
| - | - |
| 18.6s | 1.25GB |

Метрики для `python:3.13-alpine`:

| Час збірки | Розмір знімку |
| - | - |
| 20.1s | 281MB |

Бачимо, що знімок на базі Alpine збирається трохи довше ніж стандартний на базі
Debian, але при цьому займає в понад чотири рази менше місця. При цьому дельта
розміру відносно збірки без сторонніх бібліотек однакова для обох знімків.

## Golang
### 1. Створення базового Dockerfile
Базовий знімок, який було використано: `golang:1.24.3-alpine`

Dockerfile:
```dockerfile
FROM golang:1.24.3-alpine

WORKDIR /app/fizzbuzz

EXPOSE 8080

COPY . .

RUN go build -o build/fizzbuzz

CMD ["./build/fizzbuzz", "serve"]
```

Перед збіркою базовий знімок було завантажено:
```bash
docker pull golang:1.24.3-alpine
```

Метрики:

| Час збірки | Розмір знімку |
| - | - |
| 12.4s | 372MB |

Варто зазначити, що окрім того, що в базовій імплементації Dockerfile завжди
копіюється весь сирцевий код програми, також використовується знімок, який
включає в себе весь тулчейн Go, що для виконання зібраного бінарнику аж ніяк не
потрібно.

### 2. Багатоетапна збірка
Для багатоетапної збірки було додано новий етап на основі знімку `scratch`,
який є найменшим можливим знімком в Docker, від якого походять всі інші знімки.

До того ж, хоч Go самостійно встановлює потрібні залежності, для оптимізації
процесу збірки знімка можна зробити це перед компіляцією, уникаючи потреби у
повторному встановленні залежностей при перезбірці за рахунок кешування.

Початкова імплементація Dockerfile:
```dockerfile
FROM golang:1.24.3-alpine AS builder

WORKDIR /build

COPY . .

RUN go mod download
RUN go build -o fizzbuzz

FROM scratch

WORKDIR /app

COPY --from=builder /build/fizzbuzz .

EXPOSE 8080

CMD ["/app/fizzbuzz", "serve"]
```

Втім, якщо запустити контейнер на основі цього Dockerfile, ми отримаємо помилку:
```
panic: open templates/index.html: no such file or directory
```

Причина в тому, що ми покидаємо не лише файли, які потрібні для збірки, а й
файли, які потрібні під час виконання програми.

Отже, потрібно також копіювати файли, потрібні під час виконання:
```dockerfile
COPY --from=builder /build/templates ./templates
```

Метрики:

| Час збірки | Розмір знімку |
| - | - |
| 14.6s | 12.5MB |

Однак, варто зазначити, що знімок `scratch` не містить ніяких вбудованих
утиліт, через що, окрім виконання бінарнику, в ньому неможливо робити щось
більше, накшталт дебагінгу.

### 3. Distroless
Базовий знімок, який було використано: `distroless/static:nonroot`

Dockerfile:
```dockerfile
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
```

Метрики:

| Час збірки | Розмір знімку |
| - | - |
| 15.6s | 14.5MB |

Знімки `distroless` мають кілька значних переваг над `scratch`:
* мають базову файлову структуру
* підтримують часові пояси
* мають CA сертифікати, які використовуються для роботи по HTTPS
* використовують nonroot користувача
