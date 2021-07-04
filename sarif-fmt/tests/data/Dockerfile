FROM debian
RUN export node_version="0.10" \
  && apt-get update && apt-get -y install nodejs="$node_verion"
COPY package.json usr/src/app
RUN cd /usr/src/app \
  && npm install node-static

EXPOSE 80000
CMD ["npm", "start"]