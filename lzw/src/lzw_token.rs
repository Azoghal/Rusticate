// To facilitate non-char features having their own codes
// For example the end and clear code
// Or non-text files e.g. images

enum LzwToken {
    END,
    CLEAR,
}

struct Token<T> {
    value: T,
}
