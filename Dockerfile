FROM node:24.6.0-alpine
WORKDIR /usr/src/app

COPY package.json package-lock.json ./
RUN npm ci

COPY . .

EXPOSE 3000

CMD ["node", "src/index.ts"]
