FROM node:24.3.0-alpine3.21
WORKDIR /app

COPY package.json package-lock.json  ./
RUN npm --loglevel verbose install

CMD ["npm", "run", "dev"]
