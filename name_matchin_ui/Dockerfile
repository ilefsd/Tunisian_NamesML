# Stage 1: Build the Angular application
# Use a specific Node.js version for consistency
FROM node:18-alpine AS build

# Set the working directory
WORKDIR /usr/src/app

# Copy package.json and package-lock.json to leverage Docker cache
COPY package*.json ./
RUN npm install

# Copy the rest of the application source code
COPY . .

# Build the application for production
# The output will be in the /usr/src/app/dist/name_matchin_ui directory
RUN npm run build -- --configuration production

# Stage 2: Serve the application with Nginx
# Use a lightweight Nginx image
FROM nginx:1.25-alpine

# Copy the build output from the 'build' stage
# Make sure the 'name_matchin_ui' matches the project name in your angular.json
COPY --from=build /usr/src/app/dist/name_matchin_ui /usr/share/nginx/html

# Copy a custom Nginx configuration file (optional, but recommended)
# This ensures that Angular routing works correctly.
# You will need to create this nginx.conf file.
COPY nginx.conf /etc/nginx/conf.d/default.conf

# Expose port 80
EXPOSE 80
