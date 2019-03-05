function typeOf(obj) {
    if (obj['typeOf']) {
        return obj.typeOf() + ';';
    } else {
        return (typeof obj);
    }
}

function getJavaSig(args) {
    return args.map(function(arg) {
        const type = typeOf(arg);
        switch (type) {
            case 'number':
                return 'I';
            case 'string':
                return 'Ljava/lang/String;';
            case 'object':
                return 'Ljava/lang/Object;';
            default:
                return typeof type;
        }
    }).join('');
}
