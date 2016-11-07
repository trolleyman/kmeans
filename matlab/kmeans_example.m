load fisheriris;
data = meas(:,3:4);
n = size(data);
n = n(1);
x = data(:,1);
y = data(:,2);

% Render the example data
figure
scatter(x,y,10,'red','filled');
xlabel('X');
ylabel('Y');

k = 2;
% Get the starting cluster points by choosing k random numbers
%{
clusters = double.empty(0,2);
i = 1;
while i <= k
    r = randi(n);
    if any(data(r,:) == clusters)
        % We don't want two clusters to be the same, choose another random
        % number
        continue
    end
    clusters(i,:) = data(r,:);
    i = i + 1;
end
%}

% Use this for now
clusters = [6.3 1.8;4.0 1.2];

% Make a new figure with the data, as well as the cluster points
figure
hold on
scatter(x,y,10,'red','filled');
scatter(clusters(:,1), clusters(:,2), 100, 'x','blue');
hold off
xlabel('X');
ylabel('Y');
