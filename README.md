# `dfplot`

Summarize a data frame by plotting.

## Plots for numeral data

### Scatter plot

```csv
g,x,y
a,1,1
a,2,2
a,3,4
a,4,8
b,1,1
b,2,2
b,3,3
b,4,4
```

```bash
dfplot scatter data.csv --group g -x x -y y
```

### Histogram

```csv
x
1
2
2
3
3
3
4
4
4
4
```

```bash
dfplot histogram data.csv -x x
```

### Box plot

```csv
y
1
2
2
3
3
3
4
4
4
4
```

```bash
dfplot box data.csv -y y
```

## Plots for categorical data

### Bar plot

```csv
app_type,homeownership,count
individual,rent,3496
individual,mortgage,3839
individual,own,1170
joint,rent,362
joint,mortgage,950
joint,own,1170
```

```bash
# barmode = `group`
dfplot bar data.csv --group app_type -x homeownership -y count
# barmode = `proportion`
dfplot bar data.csv --group app_type -x homeownership -y count --barmode proportion
```
