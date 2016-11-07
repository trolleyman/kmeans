load fisheriris;
data = meas(:,3:4);
figure
scatter(data(:,1),data(:,2),10,'red','filled');
xlabel('X');
ylabel('Y');

