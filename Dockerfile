FROM node:14

COPY package.json yarn.lock ./
RUN yarn --pure-lockfile

COPY . .

RUN yarn

CMD yarn start