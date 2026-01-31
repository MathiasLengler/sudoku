import { zodResolver } from "@hookform/resolvers/zod";
import { Button, Container, Stack, TextField, type TextFieldProps } from "@mui/material";
import { useAtom } from "jotai";
import { atomWithStorage } from "jotai/utils";
import { useEffect } from "react";
import {
    useController,
    useForm,
    useFormState,
    type Control,
    type FieldError,
    type FieldPath,
    type FieldValues,
    type Path,
} from "react-hook-form";
import * as z from "zod";
import { getZodLocalStorage } from "./state/localStorageEffect";
import { TextFieldElement } from "react-hook-form-mui";
import { MyTextField } from "./components/formFragments/mui-rhf/MyTextField";

type DebugFormValuesInput = z.input<typeof debugFormValuesSchema>;
type DebugFormValuesOutput = z.output<typeof debugFormValuesSchema>;

const splitStringCodec = z.codec(z.string().min(1), z.array(z.string().min(1)), {
    encode: (arr) => arr.join(" "),
    decode: (str) => str.split(" "),
});

const debugFormValuesSchema = z.object({
    testString: splitStringCodec,
});

const DEBUG_FORM_DEFAULT_VALUES_OUTPUT = {
    testString: ["default"],
} satisfies DebugFormValuesOutput;

const debugFormValuesState = atomWithStorage<DebugFormValuesOutput>(
    "DebugFormValues",
    DEBUG_FORM_DEFAULT_VALUES_OUTPUT,
    getZodLocalStorage(debugFormValuesSchema),
);

export function DebugForm() {
    const [debugFormValues, setDebugFormValues] = useAtom(debugFormValuesState);

    const {
        control,
        handleSubmit,
        watch,
        formState: { isSubmitting, errors },
    } = useForm({
        values: debugFormValuesSchema.encode(debugFormValues),
        resolver: zodResolver(debugFormValuesSchema),
    });
    const watchValues = watch();
    useEffect(() => {
        console.debug("watchValues", watchValues);
    }, [watchValues]);
    useEffect(() => {
        console.debug("errors", errors);
    }, [errors]);

    return (
        <>
            <Container maxWidth="sm">
                <h1>Debug Form</h1>
                <form
                    id="generate-form"
                    onSubmit={handleSubmit(async (submitValues) => {
                        console.debug("submitValues", submitValues);
                        await new Promise((resolve) => setTimeout(resolve, 500));
                        setDebugFormValues(submitValues);
                    })}
                >
                    <Stack spacing={2}>
                        {/* <input {...register("test")} />
                        {errors.test && <span>Error: {errors.test.message}</span>} */}

                        <MyTextField control={control} name="testString" />

                        {/* Does not work, only generic over TFieldValues, not TTransformedValues */}
                        {/* <TextFieldElement control={control} name="test" /> */}

                        <Button type="submit" disabled={isSubmitting} variant="contained" color="primary">
                            Submit
                        </Button>
                    </Stack>
                </form>
            </Container>
        </>
    );
}
