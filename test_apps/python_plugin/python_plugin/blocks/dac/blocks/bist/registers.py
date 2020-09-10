##########################################################################################
# Any registers defined here will be added to this DUT and all of its derivative children
##########################################################################################

# Example of a simple register definition with all bits r/w, 0x0 is the local offset address:
#
#     SimpleReg("my_reg1", 0x0, size=32)  # 32 is the default size if not specified
SimpleReg("my_bist_reg1", 0x0, size=32)  # 32 is the default size if not specified
#
# Example of a richer definition with bitfields:
#
#     with Reg("my_reg2", 0x4):
#         Field("coco", offset=7, access="ro")
#         Field("aien", offset=6)
#         Field("diff", offset=5)
#         Field(
#             "adch",
#             offset=0,
#             width=5,
#             reset=0x1F,
#             enums={
#                 # A simple enum
#                 "val1": 3,
#                 # A more complex enum, all fields except for value are optional
#                 "val2": {
#                     "value": 5,
#                     "usage": "w",
#                     "description": "The value of something"
#                 },
#             })
#
# For more examples and full documentation see: https://origen-sdk.org/o2/guides/registers
    
