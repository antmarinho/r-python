var value = (1, "ok");
var result = 0;

match value of
    case (number, _):
        result = number + 1;
end