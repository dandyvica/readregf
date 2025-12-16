""" for line in open("copy_from_gh.txt"):
    tab = line.strip().split("|")

    type = int(tab[1])
    field = tab[2].replace(" ", "_").lower()
    comment = tab[4]
   
    print(f"// {comment}")
    match type:
        case 4: print(f"   {field}: u32,")
        case 8: print(f"   {field}: u64,")
        case _: print(f"   {field}: [u8;{type}],")

    print() """

for line in open("copy_from_gh.txt"):
    tab = line.strip().split("|")
    field = [x.strip(" )").replace(" ", "") for x in tab[0].split("(")]
    print(f"\"{field[1]}\" => CellType::{field[0]},")
    