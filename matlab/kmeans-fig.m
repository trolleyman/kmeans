% Load image
i = imread('yellowstone.jpg');
[w, h, d] = size(i);
assert(d == 3);
% Reshape image to be a vector of RGB values, and remove duplicates
j = unique(reshape(i, [w*h,3]), 'rows');
% Normalize image colour vector
nj = double(j) / 255.0;
% Seperate r, g and b channels
r = nj(:,1);
g = nj(:,2);
b = nj(:,3);

% Render scatter graph
scatter3(r, g, b, 1.0, nj);
xlabel('Red');
ylabel('Green');
zlabel('Blue');
