import { Code as MantineCode } from "@mantine/core";

type CodeProps = {
    children: string;
    wrap?: boolean;
};

export function Code({ children, wrap = false }: CodeProps) {
    return (
        <MantineCode
            block
            style={{
                minHeight: "1lh",
                whiteSpace: wrap ? "pre-wrap" : "pre",
                overflowWrap: "break-word",
                overflowX: "auto",
            }}
        >
            {children}
        </MantineCode>
    );
}
