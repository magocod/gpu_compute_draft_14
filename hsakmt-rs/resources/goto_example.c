#include <stdio.h>

int main()
{
    int a = 1;

    printf("Hello World\n");

    if (a == 2) {
        goto jump;
    }

    printf("continue World\n");


jump:
    printf("jump World\n");


    return 0;
}