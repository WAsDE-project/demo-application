FROM node:12.13.1-alpine

COPY . .

RUN npm ci

ENTRYPOINT ["npm", "start"]