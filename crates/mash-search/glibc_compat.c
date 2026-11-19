/*
 * The prebuilt ONNX Runtime library Mash links was compiled against glibc
 * 2.38, whose headers send strtol and its relatives to C23 entry points that
 * older glibc does not have. Defining them here keeps a release binary
 * loadable on glibc 2.34. The C23 versions differ only in accepting a "0b"
 * prefix, which ONNX Runtime does not parse.
 *
 * The functions are declared by hand because including <stdlib.h> on a new
 * glibc would redirect the calls below back to these same names.
 */
typedef struct __locale_struct *locale_t;

long strtol(const char *, char **, int);
long long strtoll(const char *, char **, int);
unsigned long strtoul(const char *, char **, int);
unsigned long long strtoull(const char *, char **, int);
long long strtoll_l(const char *, char **, int, locale_t);
unsigned long long strtoull_l(const char *, char **, int, locale_t);

long __isoc23_strtol(const char *text, char **end, int base) {
    return strtol(text, end, base);
}

long long __isoc23_strtoll(const char *text, char **end, int base) {
    return strtoll(text, end, base);
}

unsigned long __isoc23_strtoul(const char *text, char **end, int base) {
    return strtoul(text, end, base);
}

unsigned long long __isoc23_strtoull(const char *text, char **end, int base) {
    return strtoull(text, end, base);
}

long long __isoc23_strtoll_l(const char *text, char **end, int base, locale_t locale) {
    return strtoll_l(text, end, base, locale);
}

unsigned long long __isoc23_strtoull_l(const char *text, char **end, int base, locale_t locale) {
    return strtoull_l(text, end, base, locale);
}
