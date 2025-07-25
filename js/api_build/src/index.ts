import ts, { DeclarationStatement } from "typescript";
import { readFileSync, writeFileSync, mkdirSync, existsSync } from "node:fs";


function generate(componentModelPath: string, outFile: string) {
    const content = readFileSync(componentModelPath).toString();
    const model = JSON.parse(content) as Component[]

    const resultFile = ts.createSourceFile("unused", "", ts.ScriptTarget.Latest, false, ts.ScriptKind.TSX);
    const printer = ts.createPrinter({ newLine: ts.NewLineKind.LineFeed });

    const result = printer.printNode(ts.EmitHint.Unspecified, makeComponents(model), resultFile);

    writeFileSync(outFile, result)
}

function makeComponents(modelInput: Component[]): ts.SourceFile {
    const model = modelInput.filter((component): component is StandardComponent => component.type === "standard");

    const imports = [
        ts.factory.createImportDeclaration(
            undefined,
            ts.factory.createImportClause(
                false,
                undefined,
                ts.factory.createNamedImports([
                    ts.factory.createImportSpecifier(
                        false,
                        undefined,
                        ts.factory.createIdentifier("FC")
                    ),
                    ts.factory.createImportSpecifier(
                        false,
                        undefined,
                        ts.factory.createIdentifier("JSXElementConstructor")
                    ),
                    ts.factory.createImportSpecifier(
                        false,
                        undefined,
                        ts.factory.createIdentifier("ReactElement")
                    ),
                    ts.factory.createImportSpecifier(
                        false,
                        undefined,
                        ts.factory.createIdentifier("ReactNode")
                    )
                ])
            ),
            ts.factory.createStringLiteral("react"),
            undefined
        )
    ];

    const publicDeclarations: DeclarationStatement[] = [
        ts.factory.createTypeAliasDeclaration(
            [ts.factory.createToken(ts.SyntaxKind.ExportKeyword)],
            ts.factory.createIdentifier("ElementParams"),
            [ts.factory.createTypeParameterDeclaration(
                undefined,
                ts.factory.createIdentifier("Comp"),
                ts.factory.createTypeReferenceNode(
                    ts.factory.createIdentifier("FC"),
                    [ts.factory.createKeywordTypeNode(ts.SyntaxKind.AnyKeyword)]
                ),
                undefined
            )],
            ts.factory.createConditionalTypeNode(
                ts.factory.createTypeReferenceNode(
                    ts.factory.createIdentifier("Comp"),
                    undefined
                ),
                ts.factory.createTypeReferenceNode(
                    ts.factory.createIdentifier("FC"),
                    [ts.factory.createInferTypeNode(ts.factory.createTypeParameterDeclaration(
                        undefined,
                        ts.factory.createIdentifier("Params"),
                        undefined,
                        undefined
                    ))]
                ),
                ts.factory.createTypeReferenceNode(
                    ts.factory.createIdentifier("Params"),
                    undefined
                ),
                ts.factory.createKeywordTypeNode(ts.SyntaxKind.NeverKeyword)
            )
        ),
        ts.factory.createTypeAliasDeclaration(
            [ts.factory.createToken(ts.SyntaxKind.ExportKeyword)],
            ts.factory.createIdentifier("Element"),
            [ts.factory.createTypeParameterDeclaration(
                undefined,
                ts.factory.createIdentifier("Comp"),
                ts.factory.createTypeReferenceNode(
                    ts.factory.createIdentifier("FC"),
                    [ts.factory.createKeywordTypeNode(ts.SyntaxKind.AnyKeyword)]
                ),
                undefined
            )],
            ts.factory.createTypeReferenceNode(
                ts.factory.createIdentifier("ReactElement"),
                [
                    ts.factory.createTypeReferenceNode(
                        ts.factory.createIdentifier("ElementParams"),
                        [ts.factory.createTypeReferenceNode(
                            ts.factory.createIdentifier("Comp"),
                            undefined
                        )]
                    ),
                    ts.factory.createTypeReferenceNode(
                        ts.factory.createIdentifier("JSXElementConstructor"),
                        [ts.factory.createTypeReferenceNode(
                            ts.factory.createIdentifier("ElementParams"),
                            [ts.factory.createTypeReferenceNode(
                                ts.factory.createIdentifier("Comp"),
                                undefined
                            )]
                        )]
                    )
                ]
            )
        ),
        ts.factory.createTypeAliasDeclaration(
            [ts.factory.createToken(ts.SyntaxKind.ExportKeyword)],
            ts.factory.createIdentifier("StringNode"),
            undefined,
            ts.factory.createUnionTypeNode([
                ts.factory.createKeywordTypeNode(ts.SyntaxKind.StringKeyword),
                ts.factory.createKeywordTypeNode(ts.SyntaxKind.NumberKeyword)
            ])
        ),
        ts.factory.createTypeAliasDeclaration(
            [ts.factory.createToken(ts.SyntaxKind.ExportKeyword)],
            ts.factory.createIdentifier("EmptyNode"),
            undefined,
            ts.factory.createUnionTypeNode([
                ts.factory.createKeywordTypeNode(ts.SyntaxKind.BooleanKeyword),
                ts.factory.createLiteralTypeNode(ts.factory.createNull()),
                ts.factory.createKeywordTypeNode(ts.SyntaxKind.UndefinedKeyword)
            ])
        ),
        ts.factory.createTypeAliasDeclaration(
            [ts.factory.createToken(ts.SyntaxKind.ExportKeyword)],
            ts.factory.createIdentifier("ElementComponent"),
            [ts.factory.createTypeParameterDeclaration(
                undefined,
                ts.factory.createIdentifier("Comp"),
                ts.factory.createTypeReferenceNode(
                    ts.factory.createIdentifier("FC"),
                    [ts.factory.createKeywordTypeNode(ts.SyntaxKind.AnyKeyword)]
                ),
                undefined
            )],
            ts.factory.createUnionTypeNode([
                ts.factory.createTypeReferenceNode(
                    ts.factory.createIdentifier("Element"),
                    [ts.factory.createTypeReferenceNode(
                        ts.factory.createIdentifier("Comp"),
                        undefined
                    )]
                ),
                ts.factory.createTypeReferenceNode(
                    ts.factory.createIdentifier("EmptyNode"),
                    undefined
                ),
                ts.factory.createTypeReferenceNode(
                    ts.factory.createIdentifier("Iterable"),
                    [ts.factory.createTypeReferenceNode(
                        ts.factory.createIdentifier("ElementComponent"),
                        [ts.factory.createTypeReferenceNode(
                            ts.factory.createIdentifier("Comp"),
                            undefined
                        )]
                    )]
                )
            ])
        ),
        ts.factory.createTypeAliasDeclaration(
            [ts.factory.createToken(ts.SyntaxKind.ExportKeyword)],
            ts.factory.createIdentifier("StringComponent"),
            undefined,
            ts.factory.createUnionTypeNode([
                ts.factory.createTypeReferenceNode(
                    ts.factory.createIdentifier("StringNode"),
                    undefined
                ),
                ts.factory.createTypeReferenceNode(
                    ts.factory.createIdentifier("EmptyNode"),
                    undefined
                ),
                ts.factory.createTypeReferenceNode(
                    ts.factory.createIdentifier("Iterable"),
                    [ts.factory.createTypeReferenceNode(
                        ts.factory.createIdentifier("StringComponent"),
                        undefined
                    )]
                )
            ])
        ),
        ts.factory.createTypeAliasDeclaration(
            [ts.factory.createToken(ts.SyntaxKind.ExportKeyword)],
            ts.factory.createIdentifier("StringOrElementComponent"),
            [ts.factory.createTypeParameterDeclaration(
                undefined,
                ts.factory.createIdentifier("Comp"),
                ts.factory.createTypeReferenceNode(
                    ts.factory.createIdentifier("FC"),
                    [ts.factory.createKeywordTypeNode(ts.SyntaxKind.AnyKeyword)]
                ),
                undefined
            )],
            ts.factory.createUnionTypeNode([
                ts.factory.createTypeReferenceNode(
                    ts.factory.createIdentifier("StringNode"),
                    undefined
                ),
                ts.factory.createTypeReferenceNode(
                    ts.factory.createIdentifier("EmptyNode"),
                    undefined
                ),
                ts.factory.createTypeReferenceNode(
                    ts.factory.createIdentifier("Element"),
                    [ts.factory.createTypeReferenceNode(
                        ts.factory.createIdentifier("Comp"),
                        undefined
                    )]
                ),
                ts.factory.createTypeReferenceNode(
                    ts.factory.createIdentifier("Iterable"),
                    [ts.factory.createTypeReferenceNode(
                        ts.factory.createIdentifier("StringOrElementComponent"),
                        [ts.factory.createTypeReferenceNode(
                            ts.factory.createIdentifier("Comp"),
                            undefined
                        )]
                    )]
                )
            ])
        )
    ];

    const root = modelInput.find((component): component is RootComponent => component.type === "root");
    if (root != null) {
        for (const [name, sharedType] of Object.entries(root.sharedTypes)) {

            switch (sharedType.type) {
                case "enum": {
                    const declaration = ts.factory.createEnumDeclaration(
                        [ts.factory.createToken(ts.SyntaxKind.ExportKeyword)],
                        ts.factory.createIdentifier(name),
                        sharedType.items.map(value => {
                            return ts.factory.createEnumMember(
                                ts.factory.createIdentifier(value),
                                ts.factory.createStringLiteral(value)
                            )
                        })
                    );

                    publicDeclarations.push(declaration)
                    break;
                }
                case "object": {
                    const declaration = ts.factory.createTypeAliasDeclaration(
                        [ts.factory.createToken(ts.SyntaxKind.ExportKeyword)],
                        ts.factory.createIdentifier(name),
                        undefined,
                        ts.factory.createTypeLiteralNode(
                            Object.entries(sharedType.items).map(([propName, type]) => {
                                return ts.factory.createPropertySignature(
                                    undefined,
                                    ts.factory.createIdentifier(propName),
                                    undefined,
                                    makeType(type, "no")
                                )
                            })
                        )
                    )

                    publicDeclarations.push(declaration)
                    break;
                }
                case "union": {
                    const declaration = ts.factory.createTypeAliasDeclaration(
                        [ts.factory.createToken(ts.SyntaxKind.ExportKeyword)],
                        ts.factory.createIdentifier(name),
                        undefined,
                        ts.factory.createUnionTypeNode(
                            sharedType.items.map(type => makeType(type, "no"))
                        )
                    )

                    publicDeclarations.push(declaration)
                    break;
                }
                default: {
                    throw new Error("unreachable");
                }
            }

        }
    } else {
        throw new Error("unreachable");
    }


    const internalDeclarations = [
        ts.factory.createModuleDeclaration(
            [ts.factory.createToken(ts.SyntaxKind.DeclareKeyword)],
            ts.factory.createIdentifier("global"),
            ts.factory.createModuleBlock([ts.factory.createModuleDeclaration(
                undefined,
                ts.factory.createIdentifier("JSX"),
                ts.factory.createModuleBlock([ts.factory.createInterfaceDeclaration(
                    undefined,
                    ts.factory.createIdentifier("IntrinsicElements"),
                    undefined,
                    undefined,
                    model.map(component => {
                        return ts.factory.createPropertySignature(
                            undefined,
                            ts.factory.createComputedPropertyName(ts.factory.createStringLiteral(`gauntlet:${component.internalName}`)),
                            undefined,
                            ts.factory.createTypeLiteralNode(
                                makePropertyTypes(component, true)
                            )
                        )
                    })
                )]),
                ts.NodeFlags.Namespace | ts.NodeFlags.ExportContext | ts.NodeFlags.ContextFlags
            )]),
            ts.NodeFlags.ExportContext | ts.NodeFlags.GlobalAugmentation | ts.NodeFlags.ContextFlags
        )
    ]

    // abuse the fact that there is no space between multiline comment and content
    // is three a better way to add @internal tag to 'declare global'?
    for (const internalDeclaration of internalDeclarations) {
        ts.addSyntheticLeadingComment(internalDeclaration, ts.SyntaxKind.MultiLineCommentTrivia, "*@internal", true)
    }

    const components = model.flatMap(component => {

        const properties = component.props
            .map(prop => {
                if (!isInProperty(prop.type)) {
                    return null
                }

                return ts.factory.createJsxAttribute(
                    ts.factory.createIdentifier(prop.name),
                    ts.factory.createJsxExpression(
                        undefined,
                        ts.factory.createPropertyAccessExpression(
                            ts.factory.createIdentifier("props"),
                            ts.factory.createIdentifier(prop.name)
                        )
                    )
                )
            })
            .filter((prop): prop is ts.JsxAttribute => prop != null);

        const children = []

        const componentProps = component.props.filter(prop => !isInProperty(prop.type));
        if (componentProps.length !== 0) {
            children.push(
                ...componentProps.map(prop => (
                    ts.factory.createAsExpression(
                        ts.factory.createPropertyAccessExpression(
                            ts.factory.createIdentifier("props"),
                            ts.factory.createIdentifier(prop.name)
                        ),
                        ts.factory.createKeywordTypeNode(ts.SyntaxKind.AnyKeyword)
                    )
                ))
            );
        }

        if (component.children.type != "none") {
            children.push(ts.factory.createPropertyAccessExpression(
                ts.factory.createIdentifier("props"),
                ts.factory.createIdentifier("children")
            ))
        }

        const componentFCType = ts.factory.createTypeReferenceNode(
            ts.factory.createIdentifier("FC"),
            (properties.length === 0 && component.children.type == "none") ? [] : [
                ts.factory.createTypeReferenceNode(
                    ts.factory.createIdentifier(`${component.name}Props`),
                    undefined
                )
            ]
        )

        let componentType: ts.TypeReferenceNode | ts.IntersectionTypeNode;
        if (component.children.type == "members" || component.children.type == "string_or_members") {
            const members = { ...component.children.ordered_members, ...component.children.per_type_members }
            componentType = ts.factory.createIntersectionTypeNode([
                componentFCType,
                ts.factory.createTypeLiteralNode(
                    Object.entries(members).map(([memberName, member]) => {
                        return ts.factory.createPropertySignature(
                            undefined,
                            ts.factory.createIdentifier(memberName),
                            undefined,
                            ts.factory.createTypeQueryNode(
                                ts.factory.createIdentifier(member.componentName),
                                undefined
                            )
                        );
                    })
                )
            ]);
        } else {
            componentType = componentFCType;
        }


        let memberAssignments: ts.Statement[];
        switch (component.children.type) {
            case "string_or_members":
            case "members": {
                const members = { ...component.children.ordered_members, ...component.children.per_type_members }
                memberAssignments = Object.entries(members).map(([memberName, member]) => {
                    return ts.factory.createExpressionStatement(ts.factory.createBinaryExpression(
                        ts.factory.createPropertyAccessExpression(
                            ts.factory.createIdentifier(component.name),
                            ts.factory.createIdentifier(memberName)
                        ),
                        ts.factory.createToken(ts.SyntaxKind.EqualsToken),
                        ts.factory.createIdentifier(member.componentName)
                    ))
                });
                break;
            }
            case "string": {
                memberAssignments = [];
                break;
            }
            case "none": {
                memberAssignments = [];
                break;
            }
            default: {
                throw new Error("unreachable")
            }
        }

        const interfaceProps = makePropertyTypes(component, false);

        const interfaceDeclaration: ts.InterfaceDeclaration[] = interfaceProps.length === 0 ? [] : [
            ts.factory.createInterfaceDeclaration(
                [ts.factory.createToken(ts.SyntaxKind.ExportKeyword)],
                ts.factory.createIdentifier(`${component.name}Props`),
                undefined,
                undefined,
                interfaceProps
            )
        ];

        const propsParameter = (properties.length === 0 && component.children.type == "none") ? [] : [
            ts.factory.createParameterDeclaration(
                undefined,
                undefined,
                ts.factory.createIdentifier("props"),
                undefined,
                ts.factory.createTypeReferenceNode(
                    ts.factory.createIdentifier(`${component.name}Props`),
                    undefined
                ),
                undefined
            )
        ];

        return [
            ...interfaceDeclaration,
            ts.factory.createVariableStatement(
                [ts.factory.createToken(ts.SyntaxKind.ExportKeyword)],
                ts.factory.createVariableDeclarationList(
                    [ts.factory.createVariableDeclaration(
                        ts.factory.createIdentifier(component.name),
                        undefined,
                        componentType,
                        ts.factory.createArrowFunction(
                            undefined,
                            undefined,
                            propsParameter,
                            ts.factory.createTypeReferenceNode(
                                ts.factory.createIdentifier("ReactNode"),
                                undefined
                            ),
                            ts.factory.createToken(ts.SyntaxKind.EqualsGreaterThanToken),
                            ts.factory.createBlock(
                                [
                                    ts.factory.createReturnStatement(
                                        ts.factory.createJsxElement(
                                            ts.factory.createJsxOpeningElement(
                                                ts.factory.createJsxNamespacedName(
                                                    ts.factory.createIdentifier("gauntlet"),
                                                    ts.factory.createIdentifier(component.internalName)
                                                ),
                                                undefined,
                                                ts.factory.createJsxAttributes(properties)
                                            ),
                                            children.map(value => (
                                                ts.factory.createJsxExpression(
                                                    undefined,
                                                    value
                                                )
                                            )),
                                            ts.factory.createJsxClosingElement(ts.factory.createJsxNamespacedName(
                                                ts.factory.createIdentifier("gauntlet"),
                                                ts.factory.createIdentifier(component.internalName)
                                            ))
                                        )
                                    )
                                ],
                                true
                            )
                        )
                    )],
                    ts.NodeFlags.Const
                )
            ),
            ...memberAssignments
        ]
    });

    return ts.factory.createSourceFile(
        [...imports, ...internalDeclarations, ...publicDeclarations, ...components],
        ts.factory.createToken(ts.SyntaxKind.EndOfFileToken),
        ts.NodeFlags.None
    )
}

function makePropertyTypes(component: StandardComponent, componentPropsInChildren: boolean): ts.TypeElement[] {
    const props = component.props
        .filter(property => !isInProperty(property.type) ? !componentPropsInChildren : true)
        .map(property => {
            return ts.factory.createPropertySignature(
                undefined,
                ts.factory.createIdentifier(property.name),
                property.optional == "no" ? undefined : ts.factory.createToken(ts.SyntaxKind.QuestionToken),
                makeType(property.type, property.optional)
            )
        });

    let additionalComponentRefs: ComponentRef[];
    if (componentPropsInChildren) {
        additionalComponentRefs = component.props
            .map(property => property.type)
            .flatMap(type => collectAllComponentRefs(type));
    } else {
        additionalComponentRefs = [];
    }

    if (component.children.type != "none" || additionalComponentRefs.length > 0) {
        props.unshift(ts.factory.createPropertySignature(
            undefined,
            ts.factory.createIdentifier("children"),
            ts.factory.createToken(ts.SyntaxKind.QuestionToken),
            makeChildrenType(component.children, additionalComponentRefs)
        ))
    }

    return props
}


function makeChildrenType(type: Children, additionalComponentRefs: ComponentRef[]): ts.TypeNode {
    switch (type.type) {
        case "members": {
            const members = { ...type.ordered_members, ...type.per_type_members }

            return ts.factory.createTypeReferenceNode(
                ts.factory.createIdentifier("ElementComponent"),
                [
                    ts.factory.createUnionTypeNode(
                        [...additionalComponentRefs, ...Object.values(members)].map(member => (
                            ts.factory.createTypeQueryNode(
                                ts.factory.createIdentifier(member.componentName),
                                undefined
                            )
                        ))
                    )
                ]
            )
        }
        case "string_or_members": {
            const members = { ...type.ordered_members, ...type.ordered_members }

            return ts.factory.createTypeReferenceNode(
                ts.factory.createIdentifier("StringOrElementComponent"),
                [
                    ts.factory.createUnionTypeNode(
                        [...additionalComponentRefs, ...Object.values(members)].map(member => (
                            ts.factory.createTypeQueryNode(
                                ts.factory.createIdentifier(member.componentName),
                                undefined
                            )
                        ))
                    )
                ]
            )
        }
        case "string": {
            return ts.factory.createTypeReferenceNode(
                ts.factory.createIdentifier("StringComponent"),
                undefined
            )
        }
        case "none": {
            if (additionalComponentRefs.length > 0) {
                return ts.factory.createTypeReferenceNode(
                    ts.factory.createIdentifier("ElementComponent"),
                    [
                        ts.factory.createUnionTypeNode(
                            additionalComponentRefs.map(member => (
                                ts.factory.createTypeQueryNode(
                                    ts.factory.createIdentifier(member.componentName),
                                    undefined
                                )
                            ))
                        )
                    ]
                )
            } else {
                throw new Error("Cannot construct none children")
            }
        }
    }
}


function makeType(type: PropertyType, optional: Property["optional"]): ts.TypeNode {
    let result: ts.TypeNode
    switch (type.type) {
        case "boolean": {
            result = ts.factory.createKeywordTypeNode(ts.SyntaxKind.BooleanKeyword)
            break;
        }
        case "number": {
            result = ts.factory.createKeywordTypeNode(ts.SyntaxKind.NumberKeyword)
            break;
        }
        case "string": {
            result = ts.factory.createKeywordTypeNode(ts.SyntaxKind.StringKeyword)
            break;
        }
        case "function": {
            const params = type.arguments.map(arg => {
                if (arg.optional != "no" && arg.optional != "yes") {
                    throw new Error("following optional type is not supported here: " + arg.optional)
                }

                return ts.factory.createParameterDeclaration(
                    undefined,
                    undefined,
                    ts.factory.createIdentifier(arg.name),
                    undefined,
                    arg.optional == "no" ? makeType(arg.type, "no") : ts.factory.createUnionTypeNode([makeType(arg.type, arg.optional), ts.factory.createLiteralTypeNode(ts.factory.createNull())]),
                    undefined
                )
            });

            result = ts.factory.createFunctionTypeNode(
                undefined,
                params,
                ts.factory.createKeywordTypeNode(ts.SyntaxKind.VoidKeyword)
            )
            break;
        }
        case "component": {
            result = ts.factory.createTypeReferenceNode(
                ts.factory.createIdentifier("ElementComponent"),
                [
                    ts.factory.createTypeQueryNode(
                        ts.factory.createIdentifier(type.reference.componentName),
                        undefined
                    )
                ]
            )
            break;
        }
        case "array": {
            result = ts.factory.createArrayTypeNode(makeType(type.item, "no"))
            break;
        }
        case "shared_type_ref": {
            result = ts.factory.createTypeReferenceNode(
                ts.factory.createIdentifier(type.name),
                undefined
            )
            break;
        }
        case "union": {
            result = ts.factory.createUnionTypeNode(type.items.map(value => makeType(value, "no")))
            break
        }
        default: {
            throw new Error(`unsupported type ${JSON.stringify(type)}`)
        }
    }

    if (optional == "yes_but_complicated") {
        return ts.factory.createUnionTypeNode([result, ts.factory.createLiteralTypeNode(ts.factory.createNull())]);
    } else {
        return result
    }
}

function isInProperty(propertyType: PropertyType) {
    switch (propertyType.type) {
        case "boolean": {
            return true
        }
        case "number": {
            return true
        }
        case "string": {
            return true
        }
        case "function": {
            return true // different from the rust side
        }
        case "component": {
            return false
        }
        case "array": {
            return isInProperty(propertyType.item)
        }
        case "shared_type_ref": {
            return true
        }
        case "union": {
            if (propertyType.items.every(value => isInProperty(value))) {
                return true
            } else if (propertyType.items.every(value => !isInProperty(value))) {
                return false
            } else {
                throw new Error("")
            }
        }
        default: {
            throw new Error(`unsupported type ${JSON.stringify(propertyType)}`)
        }
    }
}

function collectAllComponentRefs(propertyType: PropertyType): ComponentRef[] {
    switch (propertyType.type) {
        case "boolean": {
            return []
        }
        case "number": {
            return []
        }
        case "string": {
            return []
        }
        case "function": {
            return []
        }
        case "component": {
            return [propertyType.reference]
        }
        case "array": {
            return collectAllComponentRefs(propertyType.item)
        }
        case "shared_type_ref": {
            return []
        }
        case "union": {
            return propertyType.items.flatMap(value => collectAllComponentRefs(value))
        }
        default: {
            throw new Error(`unsupported type ${JSON.stringify(propertyType)}`)
        }
    }
}

const genDir = "../api/src/gen";
if (!existsSync(genDir)) {
    mkdirSync(genDir);
}

generate("./component_model.json", `${genDir}/components.tsx`)
