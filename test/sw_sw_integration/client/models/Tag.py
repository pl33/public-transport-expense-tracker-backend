from typing import *

from pydantic import BaseModel, Field

from .TagOption import TagOption


class Tag(BaseModel):
    """
    None model
        JSON structure

    """

    model_config = {"populate_by_name": True, "validate_assignment": True}

    id: Optional[int] = Field(validation_alias="id", default=None)

    tag_type: str = Field(validation_alias="tag_type")

    tag_key: str = Field(validation_alias="tag_key")

    tag_name: Optional[str] = Field(validation_alias="tag_name", default=None)

    tag_display_name: Optional[str] = Field(validation_alias="tag_display_name", default=None)

    uuid: Optional[str] = Field(validation_alias="uuid", default=None)

    unit: Optional[str] = Field(validation_alias="unit", default=None)

    remarks: Optional[str] = Field(validation_alias="remarks", default=None)

    options: Optional[List[Optional[TagOption]]] = Field(validation_alias="options", default=None)
