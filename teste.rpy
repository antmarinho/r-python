var data = (2, [3, 5], Just(10), "aprovado");

match data of
    case (0, _, _, _):
        print_line("caso especial: valor zero");
    case (number, [first, second], Just(radius), status):
        print_line("caso completo");
        print_line(number + first + second + radius);
        print_line(status);
    case (_, _, Nothing, status):
        print_line("sem valor interno");
        print_line(status);
    case _:
        print_line("nenhum padrão específico");
end
